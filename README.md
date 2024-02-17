# Muninn

Processes external data inputs into usable text summaries

## To build the docker file

```sh
docker build -t muninn .
```

## To run the docker file

Remember to pass in the environment variables for the API keys

```sh
docker run -e OPENAI_API_KEY=$OPENAI_API_KEY -p 8080:8080 muninn
```
