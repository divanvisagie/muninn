# Muninn

Muninn is a memory system designed to augment LLM powered chat bots with
advanced RAG based memory retrieval. It is designed to be a drop in service for
any chat bot that uses a OpenAI compatible API.

![Muninn](docs/design.png)

## To build the docker file

```sh
docker build -t muninn .
```

## To run the docker file

Remember to pass in the environment variables for the API keys

```sh
docker run -e OPENAI_API_KEY=$OPENAI_API_KEY -p 8080:8080 muninn
```
