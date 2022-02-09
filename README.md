# Chemix backend

##cmd
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
- [*] 1、api接口定位，提供假数据
- [ ] 2、撮合逻辑实现
- [*] 3、swagger的替代在线文档
- [*] 4、enther交互sdk调试
- [ ] 5、数据库字段和索引设计
- [ ] 6、定时进行数据加工的逻辑
- [ ] 7、Cargo test 测试用例
- [ ] 8、机器人
- [*] 9、actix_web_actor ws
- [ ] 10、签名验签相关
- [*] 11、撮合和取消订单的异步处理,通过状态
- [ ] 12、 ng部署https
- [ ] 13、 搭建BSC节点，压测用
- [*] 14、 部署测试合约做事件监听
- [*] 15、 基于redis的进程间通信,撮合结果推redis，推ws服务
- [ ] 16、 addmin推bsc调matchTrade和thewOrder接口
```

##Fix
```
1、apidoc 400错误码的问题
2、取消订单先挂起逻辑，等撮合结果上链之后，再处理取消
```
