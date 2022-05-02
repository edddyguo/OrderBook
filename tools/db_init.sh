#!/bin/bash
sudo -u postgres psql -d chemix_pro -U postgres -f 0001-init.down.sql
sudo -u postgres psql -d chemix_pro -U postgres -f 0001-init.up.sql
sudo -u postgres psql -d chemix_pro -U postgres -f seed.sql
##psql -U postgres -d postgres -h 127.0.0.1 -p 5432