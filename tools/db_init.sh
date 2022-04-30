#!/bin/bash
sudo -u postgres psql -d chemix_pro -U postgres -f 0001-init.down.sql
sudo -u postgres psql -d chemix_pro -U postgres -f 0001-init.up.sql
sudo -u postgres psql -d chemix_pro -U postgres -f seed.sql