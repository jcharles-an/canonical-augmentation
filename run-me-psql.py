import asyncio
import logging
import math
import queue
import subprocess
import sys
import threading
import time
import os
import traceback
from typing import List

import psycopg
from psycopg import sql, AsyncConnection
from psycopg_pool import AsyncConnectionPool

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

def log(message:str, level:int=logging.INFO, exception=None):
  if exception:
    level = logging.ERROR
    #logger.log(level, message, exc_info=exception)
    logger.log(level, message)
  else:
    logger.log(level, message)

# --- Configuration Variables (from Bash script) ---
# These would typically come from command-line arguments or a config file
N = int(sys.argv[1]) if len(sys.argv) > 1 else 7 # Default N if not provided
K = int(sys.argv[2]) if len(sys.argv) > 2 else 3  # Default K if not provided

CASE = f"{N}_{K}"
SCHEMA = f"conf_{CASE}"
MAX_GRAPHS_PER_THREAD=400
WORKER_THREADS="10"
DB_HOST = "localhost"
DB_USER = "jcharlesan"
DB_PASS = "190128" # WARNING: Storing passwords directly in code is insecure.
DB_NAME = "n_k" # The database name is 'n_k' from the bash script DBURL

# Base DB URL for psql commands (without table/schema for parallel's --sqlmaster)
# For parallel's --sqlmaster/--sqlandworker, the table name is appended after the database name.
# Example: postgresql://user:pass@host/dbname/schema.table
BASE_DB_URL = f"postgresql://{DB_USER}:{DB_PASS}@{DB_HOST}/{DB_NAME}"

EXECUTABLE = "./target/release/shcreier-sims-app"

# --- Helper Functions for Database Interaction (using psycopg) ---
conn = None
async_pool = None
def get_db_connection():
  """Establishes a psycopg database connection."""
  try:
    #global conn
    #if conn is None:
    conn = psycopg.connect(
      host=DB_HOST,
      dbname=DB_NAME,
      user=DB_USER,
      password=DB_PASS
    )
    conn.autocommit = True # Auto-commit for simple DDL/DML
    return conn
  except Exception as e:
    log(f"Error connecting to database", exception=e)
    sys.exit(1)

async def get_async_db_connection():
  """Establishes a psycopg database connection."""
  try:
    global async_pool
    if async_pool is None:
      dsn = f"postgresql://{DB_USER}:{DB_PASS}@{DB_HOST}/{DB_NAME}"
      async_pool = AsyncConnectionPool(
        dsn,
        min_size=5, max_size=20, # Adjust min/max connections as needed
      )
      # await async_pool.open()
    return async_pool.connection()
  except Exception as e:
    log(f"Error connecting to database", exception=e)
    sys.exit(1)

def execute_db_query(query_string, fetch_one=False, fetch_all=False):
  conn = get_db_connection()
  try:
    with conn.cursor() as cur:
      cur.execute(query_string)
      if fetch_one:
        return cur.fetchone()
      if fetch_all:
        return cur.fetchall()
      return None # For DDL/DML
  except Exception as e:
    log(f"Error executing query '{query_string}'", exception=e)
    raise

async def execute_async_db_query(query_string, fetch_one=False, fetch_all=False):
  async_conn = await get_async_db_connection()
  async_conn.autocommit = True
  try:
    async with async_conn as conn:
      async with conn.cursor() as cur:
        log(f"Executing async query '{query_string}'", level=logging.DEBUG)
        await cur.execute(query_string)
        if fetch_one:
          return await cur.fetchone()
        if fetch_all:
          return await cur.fetchall()
        return None # For DDL/DML
  except Exception as e:
    log(f"Error executing query '{query_string}'", exception=e)
    raise


def set_up_connection(schema_name):
  log(f"Setting up schema: {schema_name}")
  try:
    execute_db_query(sql.SQL("CREATE SCHEMA IF NOT EXISTS {};").format(
      sql.Identifier(schema_name)
    ))
    log(f"Schema '{schema_name}' ensured.")
  except Exception as e:
    log(f"Failed to create schema {schema_name}: ", exception=e)
    sys.exit(1)

def get_empty_graph(n_val):
  return ":".join(["0"] * (2 * n_val))


def tasks_have_been_sent(table_name:str):
  check_query = sql.SQL("""
                        SELECT count(*) FROM {}.{} WHERE V1 <> 'starting';
                        """).format(sql.Identifier(SCHEMA), sql.Identifier(table_name))
  count = execute_db_query(check_query, fetch_one=True)[0]
  return count > 0

async def get_total_tasks(table_name:str):
  check_query = sql.SQL("""
                        SELECT count(*) FROM {}.{};
                        """).format(sql.Identifier(SCHEMA), sql.Identifier(table_name))
  count = await execute_async_db_query(check_query, fetch_one=True)
  return count[0]

async def get_finished_tasks(table_name:str):
  check_query = sql.SQL("""
                        SELECT count(*) FROM {}.{} WHERE exitval <> -1000 AND stdout NOT LIKE 'Reserved-%';
                        """).format(sql.Identifier(SCHEMA), sql.Identifier(table_name))
  count = await execute_async_db_query(check_query, fetch_one=True)
  return count[0]

def create_line_table(table_name:str, line: int) -> List[asyncio.Future]:
  log(f"Creating table: {table_name}")
  try:
    check_query = sql.SQL("""
                          SELECT count(*) FROM information_schema.tables
                          WHERE table_schema = {} AND table_name = {};
                          """).format(sql.Literal(SCHEMA), sql.Literal(table_name))
    table_exists = execute_db_query(check_query, fetch_one=True)[0]

    table_name = f"line_{line}"
    pending_results_name = f"{table_name}_pending_results"
    channel_function_name = f"process_{table_name}_stdout"

    if table_exists == 0:
      log(f"Table '{table_name}' does not exist. Initializing with parallel --sqlmaster.")
      try:
        # Use the full URL including schema.table for --sqlmaster
        db_url_table = f"{BASE_DB_URL}/{SCHEMA}.{table_name}"
        subprocess.run(["parallel", "--sqlmaster", db_url_table, "echo", ":::", "starting"],
                       check=True,capture_output=True,  text=True )
        log(f"Parallel --sqlmaster initialized for {table_name}.")

      except subprocess.CalledProcessError as e:
        log(f"Error initializing parallel --sqlmaster for {table_name}", exception=e)
        sys.exit(1)


    else:
      log(f"Table '{table_name}' already exists. Skipping creation.")

    # Creating results table
    create_listener(line)

    results_workers=[]
    workers = 1
    for i in range(0,workers):
      thread_fraction = i / workers
      log(f"Starting worker {i+1}/{workers}")
      results_workers.append(
        asyncio.create_task(
          process_results(pending_results_name,table_name, channel_function_name,
                          thread_fraction=thread_fraction)
        )
      )

    return results_workers


  except Exception as e:
    log(f"Failed to create table {table_name}: ", exception=e)
    sys.exit(1)

def get_graph(line_num, n_val):
  if line_num == "null":
    return [get_empty_graph(n_val)]
  else:
    results_table_name = f"line_{line_num}_results"
    query_str = sql.SQL("SELECT graph FROM {}.{};").format(
      sql.Identifier(SCHEMA), sql.Identifier(results_table_name)
    )
    try:
      graph_data = execute_db_query(query_str, fetch_all=True)
      if graph_data:
        return [row[0] for row in graph_data]
      else:
        log(f"Warning: No graph found in {results_table_name}. Returning empty graph.", logging.ERROR)
        return get_empty_graph(n_val)
    except Exception as e:
      log(f"Error getting graph from {results_table_name}: ", exception=e)
      return get_empty_graph(n_val) # Fallback to empty graph on error

def create_listener(line_num:int):
  table_name = f"line_{line_num}"
  results_name = f"{table_name}_results"
  pending_results_name = f"{table_name}_pending_results"
  channel_function_name = f"process_{table_name}_stdout"

  log(f"Creating results table: {results_name}")
  try:

    function_query = sql.SQL(f"""
      CREATE TABLE IF NOT EXISTS 
        {SCHEMA}.{results_name} (
          cert NUMERIC CONSTRAINT {results_name}_pk PRIMARY KEY,
          graph TEXT
        ) PARTITION BY HASH (cert);
        """)
    execute_db_query(function_query)

    n = 1024
    for i in range(0,n):
      function_query = sql.SQL(f"""
        CREATE TABLE  IF NOT EXISTS 
          {SCHEMA}.{results_name}_p{i} PARTITION OF {SCHEMA}.{results_name}
          FOR VALUES WITH (MODULUS {n}, REMAINDER {i});
        """)
      execute_db_query(function_query)



    function_query = sql.SQL(f"""
                              CREATE TABLE IF NOT EXISTS 
                                {SCHEMA}.{results_name} (
                                  cert NUMERIC CONSTRAINT {results_name}_pk PRIMARY KEY,
                                  graph TEXT
                                 );
                                 
                              CREATE TABLE IF NOT EXISTS 
                                {SCHEMA}.{pending_results_name} (
                                  seq INTEGER CONSTRAINT {pending_results_name}_pk PRIMARY KEY
                                 );
                                 
                              CREATE OR REPLACE FUNCTION {channel_function_name}(
                                seq_id INTEGER
                              )
                              RETURNS void AS $$
                              DECLARE
                                line TEXT;
                                current_stdout TEXT;
                                solutions INTEGER := 0; -- Initialize solutions
                                certs NUMERIC[] := '{{}}';    -- Array to hold all certs
                                graphs TEXT[] := '{{}}';   -- Array to hold all graphs
                                batch_count INTEGER := 0;   -- Counter for items in current batch
                                BATCH_LIMIT INTEGER := 5000; -- <<<--- DEFINE YOUR BATCH LIMIT HERE (e.g., 1000, 5000, 10000)
                              BEGIN
                                SELECT stdout INTO current_stdout FROM {SCHEMA}.{table_name} WHERE seq = seq_id;   
                                
                                 -- Handle case where stdout is empty or null, nothing to process
                                IF current_stdout IS NULL OR TRIM(current_stdout) = '' THEN
                                  UPDATE {SCHEMA}.{table_name} SET stdout = 'Done 0' WHERE seq = seq_id;
                                  DELETE FROM {SCHEMA}.{pending_results_name} WHERE seq = seq_id;
                                  RETURN;
                                END IF;
                                
                                -- Parse all lines and collect certs and graphs into arrays, inserting in batches
                                FOR line IN SELECT * FROM unnest(string_to_array(current_stdout, E'\n')) LOOP
                                  IF TRIM(line) = '' THEN
                                    CONTINUE;
                                  END IF;
                              
                                  solutions := solutions + 1;
                                  -- Append parsed values to arrays for the current batch
                                  certs := array_append(certs, SPLIT_PART(line, ' ', 1)::NUMERIC);
                                  graphs := array_append(graphs, SUBSTRING(line FROM POSITION(' ' IN line) + 1));
                                  batch_count := batch_count + 1;
                              
                                  -- If batch limit is reached, perform the batch insert and reset arrays
                                  IF batch_count >= BATCH_LIMIT THEN
                                    INSERT INTO {SCHEMA}.{results_name} (cert, graph)
                                    SELECT c, g
                                    FROM unnest(certs, graphs) AS t(c, g) -- Unnest both arrays side-by-side
                                    ON CONFLICT (cert) DO NOTHING;
                              
                                    -- Reset arrays and count for the next batch
                                    certs := '{{}}';
                                    graphs := '{{}}';
                                    batch_count := 0;
                                  END IF;
                              
                                END LOOP;
                              
                                -- After the loop, insert any remaining items in the last (possibly partial) batch
                                IF batch_count > 0 THEN
                                  INSERT INTO {SCHEMA}.{results_name} (cert, graph)
                                  SELECT c, g
                                  FROM unnest(certs, graphs) AS t(c, g)
                                  ON CONFLICT (cert) DO NOTHING;
                                END IF;
                                  
                                UPDATE {SCHEMA}.{table_name} SET stdout = 'Done ' || solutions::text WHERE seq = seq_id;
                                DELETE FROM {SCHEMA}.{pending_results_name} WHERE seq = seq_id;
                              END;
                              $$ LANGUAGE plpgsql;
                              
                              CREATE OR REPLACE FUNCTION insert_into_{pending_results_name}()
                              RETURNS TRIGGER AS $$
                              BEGIN
                                IF NEW.stdout NOT LIKE 'Reserved-%' AND NEW.stdout NOT LIKE 'Done %' THEN
                                  INSERT INTO {SCHEMA}.{pending_results_name} (seq) VALUES (NEW.seq);
                                END IF;
                                RETURN NEW; 
                              END;
                              $$ LANGUAGE plpgsql;
                              

                              CREATE OR REPLACE TRIGGER notify_{table_name}_stdout_trigger
                                AFTER UPDATE ON {SCHEMA}.{table_name}
                                FOR EACH ROW
                                EXECUTE FUNCTION insert_into_{pending_results_name}();
                                
                              """)
    execute_db_query(function_query)



    log(f"Results tables for {line_num} created")


  except Exception as e:
    log(f"Failed to create results table {results_name}: ", exception=e)
    sys.exit(1)

async def process_results(pending_results_name:str,table_name:str, channel_function_name:str, thread_fraction: float):
  await asyncio.sleep(1)
  while True:
    try:
      await asyncio.sleep(0.1)
      row = await execute_async_db_query(f"""SELECT count(*) FROM {SCHEMA}.{pending_results_name}""", fetch_one=True)
      offset = math.floor(thread_fraction * row[0])

      row = await execute_async_db_query(f"""
                 SELECT seq
                 FROM {SCHEMA}.{pending_results_name}
                 order by 1
                   FOR UPDATE SKIP LOCKED
                 LIMIT 1 OFFSET {offset}
                 """, fetch_one=True)

      if row is not None:
        seq = row[0]
        #log(f"[Process results ID: {thread_fraction:.2f}] Processing task ID: {seq} offset: {offset}")
        await execute_async_db_query(f"SELECT {channel_function_name}({seq});", fetch_one=True)
     # else:
       # log(f"[Process results ID: {thread_fraction:.2f}] offset: {offset} found no rows")

      row = await execute_async_db_query(f"""SELECT count(*) FROM {SCHEMA}.{table_name} WHERE stdout NOT LIKE 'Done %'""", fetch_one=True)
      if row[0] == 0:
        row = await execute_async_db_query(f"""SELECT count(*) FROM {SCHEMA}.{table_name}""", fetch_one=True)
        if row[0] != 0:
          break


    except Exception as e:
      log(f"[Process results ID: {thread_fraction:.2f}] Error processing task: some excpection",exception=e)

def create_results_table(line_num):
  table_name = f"line_{line_num}"
  results_name = f"{table_name}_results"

  log(f"Creating results table: {results_name}")
  try:

    create_table_query = sql.SQL("""
                                 CREATE TABLE IF NOT EXISTS {}.{} (
                                                                    cert TEXT CONSTRAINT {} PRIMARY KEY,
                                                                    graph TEXT
                                 );
                                 """).format(
      sql.Identifier(SCHEMA), sql.Identifier(results_name), sql.Identifier(f"{results_name}_pk")
    )
    execute_db_query(create_table_query)

    # delete_query = sql.SQL("DELETE FROM {}.{} WHERE V1 = 'starting';").format(
    #   sql.Identifier(SCHEMA), sql.Identifier(table_name)
    # )
    # execute_db_query(delete_query)

    trigger_query = sql.SQL(f"""
                              CREATE OR REPLACE FUNCTION process_{table_name}_stdout()
                              RETURNS TRIGGER AS $$
                              DECLARE
                                line TEXT;
                                current_cert TEXT;
                                current_graph TEXT;
                                solutions INTEGER;
                              BEGIN
                                IF PG_TRIGGER_DEPTH() = 1 THEN
                                  IF NEW.stdout NOT LIKE 'Reserved-%' THEN
                                    solutions:=0;
                                    FOR line IN SELECT * FROM unnest(string_to_array(NEW.stdout, E'\n')) LOOP
                                      IF TRIM(line) = '' THEN
                                        CONTINUE;
                                      END IF;
                                      
                                      solutions:=solutions + 1;
                                      current_cert := SPLIT_PART(line, ' ', 1);
                                      current_graph := SUBSTRING(line FROM POSITION(' ' IN line) + 1);
      
                                      INSERT INTO {SCHEMA}.{results_name} (cert, graph)
                                      VALUES (current_cert, current_graph)
                                      ON CONFLICT (cert) DO NOTHING; 
                                    END LOOP;
                                    UPDATE {SCHEMA}.{table_name} SET stdout = solutions::text WHERE seq = NEW.seq;
                                  END IF;
                                END IF;
                                RETURN NEW; 
                              END;
                              $$ LANGUAGE plpgsql;

                              CREATE TRIGGER process_{table_name}_stdout_trigger
                                AFTER UPDATE ON {SCHEMA}.{table_name}
                                FOR EACH ROW
                                EXECUTE FUNCTION process_{table_name}_stdout();
                              """)
    execute_db_query(trigger_query)

    log(f"Results table for line {line_num} created and populated.")



  except Exception as e:
    log(f"Failed to create results table {results_name}: ", exception=e)
    sys.exit(1)



def prepare_graphs(graphs: List[str]) -> str:
  n = MAX_GRAPHS_PER_THREAD
  output = ""
  for i in range(0, len(graphs), n):
    input_str="["
    for graph in graphs[i:i + n]:
      input_str+=f"{graph},"
    input_str=input_str[:-1] + "]\n"
    output += input_str
  return output

def schedule_tasks(previous_line: int, LINE: int, db_url_table_for_parallel:str):
  graph_input = get_graph(previous_line, N)
  log(f"Graph input for line {LINE}: {graph_input[:4]}...")
  input_for_parallel = prepare_graphs(graph_input)

  # Run parallel
  log(f"Scheduling parallel tasks for line {LINE}...")
  parallel_command = [
    "parallel",
    "--sqlmaster", f"+{db_url_table_for_parallel}",
    f"{EXECUTABLE} {K} {LINE}",
  ]

  try:
    # Use Popen to pipe input to parallel
    process = subprocess.Popen(
      parallel_command,
      stdin=subprocess.PIPE,
      stdout=subprocess.PIPE,
      stderr=subprocess.PIPE,
      text=True # For string input/output
    )
    stdout, stderr = process.communicate(input=input_for_parallel)

    if process.returncode != 0:
      log(f"Scheduling parallel tasks for line {LINE}:\n{stderr}", level=logging.ERROR)
      sys.exit(1)
    else:
      log(f"Scheduling parallel tasks for line {LINE} completed.")

  except Exception as e:
    log(f"Error scheduling parallel tasks for line{LINE}: ", exception=e)
    sys.exit(1)



def read_pipe(pipe, q):
  try:
    for line in iter(pipe.readline, ''):
      q.put((pipe.name, line.strip())) # Store source (stdout/stderr) and line
  finally:
    pipe.close() # Ensure pipe is closed when thread exits


async def execute_tasks(LINE: int, db_url_table_for_parallel:str) -> None:
  parallel_command = [
    "parallel",
    "--sqlworker", db_url_table_for_parallel,
    "-j", WORKER_THREADS,
    f"{EXECUTABLE} {K} {LINE}",
  ]
  table_name = f"line_{LINE}"

  try:
    log(f"Executing parallel tasks for line {LINE}...")
    process = await asyncio.create_subprocess_exec(
      *parallel_command,
      stdout=subprocess.PIPE,
      stderr=subprocess.PIPE
    )

    stdout_collector_task = asyncio.create_task(process.stdout.read())
    stderr_collector_task = asyncio.create_task(process.stderr.read())

    tasks_number = await get_total_tasks( table_name)

    while process.returncode is None: # Loop as long as the subprocess is running
      finished_tasks = await get_finished_tasks(table_name) # Await async DB call
      percent_complete = (finished_tasks / tasks_number * 100) if tasks_number > 0 else 0
      log(f"[Execute tasks] Line {LINE} completed tasks: {finished_tasks} / {tasks_number}  %{percent_complete:.2f}")

      # 4. CRUCIAL: Yield control to the event loop.
      # This allows other async tasks (like your listener) to run.
      await asyncio.sleep(1)

    returncode = await process.wait()

    # 6. Await completion of output/error reading tasks
    stdout_output_bytes, stderr_output_bytes = await asyncio.gather(stdout_collector_task, stderr_collector_task)
    stdout_output = stdout_output_bytes.decode(errors='ignore').strip()
    stderr_output = stderr_output_bytes.decode(errors='ignore').strip()



    if returncode == 0:
      final_finished_tasks = await get_finished_tasks(table_name)
      percent_complete = (final_finished_tasks / tasks_number * 100) if tasks_number > 0 else 0
      log(f"[Execute tasks] Line {LINE} completed tasks: {final_finished_tasks} / {tasks_number}  %{percent_complete:.2f}")

      if final_finished_tasks != tasks_number:
        log(f"Expected {tasks_number} tasks, but {final_finished_tasks} finished for line {LINE}. Resetting stuck tasks.", level=logging.WARNING)
        await reset_stuck_tasks(table_name)
        log(f"Freed parallel stuck tasks for line {LINE}, reexecuting...", level=logging.INFO)
        # Recursive call to self (now fully async)
        await execute_tasks(LINE, db_url_table_for_parallel)
      else:
        log(f"Executing parallel tasks for line {LINE} completed.")
    else:
      log(f"\nExecuting parallel tasks for line {LINE} failed with code {returncode}:", level=logging.ERROR)
      if stdout_output:
        log(f"[Stdout]:\n{stdout_output}", level=logging.ERROR)
      if stderr_output:
        log(f"[Stderr]:\n{stderr_output}", level=logging.ERROR)
      else:
        log("No stdout/stderr output captured.", level=logging.ERROR)

  except Exception as e:
    log(f"Executing parallel tasks for line {LINE}", exception=e)
    sys.exit(1)


async def reset_stuck_tasks(table_name:str) -> None:
  #select_stdout_query = sql.SQL("DELETE FROM {}.{} WHERE stdout LIKE 'Reserved-%';").format(sql.Identifier(SCHEMA), sql.Identifier(table_name))
  select_stdout_query = sql.SQL('''UPDATE {}.{} 
                                   SET host = null, starttime = null, exitval = -1000, stdout = null 
                                   WHERE stdout LIKE 'Reserved-%';''').format(sql.Identifier(SCHEMA), sql.Identifier(table_name))
  await execute_async_db_query(select_stdout_query)

async def process_lines(line_limit:int):
  log("Building shcreier-sims-app...")
  try:
    global N
    os.environ["N_C"] = str(N)
    subprocess.run(["cargo", "build", "--release"],check=True)
    log("shcreier-sims-app built successfully.")
  except subprocess.CalledProcessError as e:
    log(f"Error building shcreier-sims-app: {e.stderr}", level=logging.ERROR)
    sys.exit(1)

  previous_line = "null" # Initial state for the first iteration

  for LINE in range(line_limit):
    log(f"--- Processing LINE: {LINE} ---")
    table_name = f"line_{LINE}"
    db_url_table_for_parallel = f"{BASE_DB_URL}/{SCHEMA}.{table_name}"


    workers = create_line_table(table_name, LINE)


    if not tasks_have_been_sent(table_name):
      schedule_tasks(previous_line, LINE, db_url_table_for_parallel)

    delete_query = sql.SQL("DELETE FROM {}.{} WHERE V1 = 'starting';").format(
      sql.Identifier(SCHEMA), sql.Identifier(table_name)
    )
    execute_db_query(delete_query)

    workers.append(asyncio.create_task(execute_tasks(LINE, db_url_table_for_parallel)))

    log(f"Awaiting for execute and process result tasks execution...")
    results = await asyncio.gather(*workers)

    # Create results table and populate it
    # create_results_table(LINE, N)

    previous_line = LINE # Update previous_line for next iteration

def reset_DB():
  log(f"Resetting database by dropping schema: {SCHEMA}")
  try:
    execute_db_query(sql.SQL("DROP SCHEMA IF EXISTS {} CASCADE;").format(
      sql.Identifier(SCHEMA)
    ))
    log(f"Schema '{SCHEMA}' dropped successfully.")
  except Exception as e:
    log(f"Failed to drop schema {SCHEMA}: ", exception=e)
    # Don't exit here, as it might be okay if schema didn't exist or was empty.
    # But log the error.






# --- Main Execution Block ---
if __name__ == "__main__":
  log(f"Starting script with N={N}, K={K}, Schema={SCHEMA}")

  #reset_DB()
  set_up_connection(SCHEMA)
  asyncio.run(process_lines(N))

  log("\nScript finished successfully.")

