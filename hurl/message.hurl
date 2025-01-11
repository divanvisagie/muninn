POST http://localhost:8080/api/v1/message
{
	"chat_id": 5,
    "text": "/test",
    "telegram_user_id": 192455
}
HTTP 200
[Asserts]
body contains "Hello, world"
