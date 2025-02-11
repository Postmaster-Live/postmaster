import requests

# Helper function to generate vector from an encoder API
def generate_embedding(text):
    encoder_url = "https://your-encoder-api-url"
    headers = {"Authorization": f"Bearer {your_api_key}"}
    response = requests.post(encoder_url, json={"text": text}, headers=headers)
    embedding = response.json().get('embedding', [])
    return embedding
