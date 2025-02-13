-- Create table for documents
CREATE TABLE IF NOT EXISTS DOCUMENTS (
    ID STRING PRIMARY KEY,
    FILENAME STRING,
    CONTENT STRING
);

-- Create table for paragraph analysis
CREATE TABLE IF NOT EXISTS PARAGRAPH_ANALYSIS (
    ID STRING PRIMARY KEY,
    DOCUMENT_ID STRING REFERENCES DOCUMENTS(ID),
    PARAGRAPH STRING,
    SIMPLE_QUESTIONS STRING,
    COMPLEX_QUESTIONS STRING,
    CONTEXT STRING,
    SCOPE STRING
);

-- Create Snowflake UDF for question generation
CREATE OR REPLACE FUNCTION GENERATE_QUESTIONS(TEXT STRING) 
RETURNS OBJECT 
LANGUAGE PYTHON 
RUNTIME_VERSION = '3.8' 
HANDLER = 'generate_questions'
PACKAGES = ('requests')
AS $$
import requests
import json

def generate_questions(text):
    """Call OpenAI API to generate questions, context, and scope."""
    API_URL = "https://api.openai.com/v1/chat/completions"
    HEADERS = {
        "Authorization": f"Bearer {API_KEY}",
        "Content-Type": "application/json"
    }

    prompt = f"""Extract the following from the text:
    - Simple Questions
    - Complex Questions
    - Context
    - Scope
    
    Text: {text}
    """

    data = {
        "model": "gpt-4-turbo",
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.7
    }

    response = requests.post(API_URL, headers=HEADERS, json=data)
    output = response.json()
    
    return json.loads(output["choices"][0]["message"]["content"])
$$;
