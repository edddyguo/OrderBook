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
- [ ] 1、dashboard
- [ ] 2、kline
- [ ] 3、enther交互sdk调试
- [ ] 4、定时进行数据加工的逻辑
- [ ] 5、Cargo test补充
- [ ] 6、签名验签相关
- [ ] 7、 取消订单的处理
- [ ] 8、 ssl
```
