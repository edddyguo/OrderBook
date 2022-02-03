##cmd
```
curl -X POST 'http://localhost:8000/publish' \
-H 'Content-Type: application/json' \
-d '{"user_id": 1, "topic": "cats", "message": "are awesomeeeeeeeeeeeeeee"}'

curl -X DELETE 'http://localhost:8000/register/32cc16e896554ef7b8ef7e7ff0f285eb'

curl -X POST 'http://localhost:8000/register' -H 'Content-Type: application/json' -d '{ "user_id": 1 }'
```
