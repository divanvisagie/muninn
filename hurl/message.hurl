POST http://localhost:8080/api/v1/message
{
	"chatId": 5,
    "text": "Hello, I am a chatbot!",
    "telegramUserId": 192455
}
HTTP 200
[Asserts]
body contains "Hello, world"
