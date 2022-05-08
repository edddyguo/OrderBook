# Chemix backend

##db
```
sudo -u postgres psql -d chemix -U postgres -f ./models/0001-init.up.sql
psql -U postgres -d postgres -h 127.0.0.1 -p 5432
psql -U postgres -d chemix  -h 127.0.0.1 -p 5432
```

##api doc
```
apidoc -f ./api/src/main.rs -i api/ -o doc/
```


##TODO
```
- [*] 1、order book engine feature
- [ ] 2、kline
- [ ] 3、chain fork deal with
- [ ] 4、Cargo test
```
