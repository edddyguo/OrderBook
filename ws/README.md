##cmd
```
curl -X POST 'http://localhost:8000/publish' \
-H 'Content-Type: application/json' \
-d '{"user_id": 1, "topic": "cats", "message": "are awesome"}'

curl -X DELETE 'http://localhost:8000/register/32cc16e896554ef7b8ef7e7ff0f285eb'

curl -X POST 'http://localhost:8000/register' -H 'Content-Type: application/json' -d '{ "user_id": 1 }'

ws://127.0.0.1:8000/ws/37c400d2c20c46078656df3fd2c1126b
```
