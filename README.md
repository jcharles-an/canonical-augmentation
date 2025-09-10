# Stuff I had to do to set this up in Silicon Mac

## Install psql with homebrew

Add this line to the psql configs to allow connetions from 100:

```Bash
vi /opt/homebrew/var/postgresql@15/pg_hba.conf

host    all                     all                     100.0.0.0/8                     scram-sha-256
```

Give psql permissions
```PostgreSQL
GRANT ALL PRIVILEGES ON SCHEMA n_k TO jcharlesan;

ALTER DEFAULT PRIVILEGES IN SCHEMA n_k GRANT ALL PRIVILEGES ON TABLES TO jcharlesan;
ALTER DEFAULT PRIVILEGES IN SCHEMA n_k GRANT ALL PRIVILEGES ON FUNCTIONS TO jcharlesan;
ALTER DEFAULT PRIVILEGES IN SCHEMA n_k GRANT ALL PRIVILEGES ON SEQUENCES TO jcharlesan;
```

## Install homebrew perl
Make sure bash is using homebrew perl
install cpanm (DO NOT USE SUDO AS THIS SHOULD NOT BE INSTALLED SYSTEM WIDE):
```Bash
curl -L https://cpanmin.us | perl - App::cpanminus
```
Verify that cpanm is not hard coded to the system perl (instead of hoembrew) if it is, change it to homebrew:
```Bash
head -1 /Users/jcharlesan/perl5/bin/cpanm
#!/opt/homebrew/bin/perl
```
Install using cpanm:
```Bash
PG_CONFIG=$(brew --prefix postgresql@15)/bin/pg_config cpanm DBD::Pg
```
some tests might fail due to long strings an mac shit but you can try forcing the installation and checking later on::
```Bash
PG_CONFIG=$(brew --prefix postgresql@15)/bin/pg_config /Users/jcharlesan/perl5/bin/cpanm --force DBD::Pg
perl -MDBD::Pg -e 'print "DBD::Pg is installed!\n"'
```

16 260 473 170 995 536 115
 2 968 431 146 628 994 648
