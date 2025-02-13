# **PostMaster Design Document: Snowflake + Mistral Integration for Document Processing using CVRS **  

## **1. Overview**  
Postman enables to ingests documents, extracts paragraphs, and generates metadata (simple questions, complex questions, context, and scope) using the **Mistral-Small-2409** model. The processed data is stored in **Snowflake**, and queries are optimized using a **Softmax-based ranking** mechanism as described in the referenced paper.  

## **2. System Architecture**  
- **Flask API**: Handles ingestion and querying of documents.  
- **Snowflake Database**: Stores extracted paragraphs and metadata.  
- **Mistral-Small-2409**: Generates metadata for each paragraph.  
- **UDF (Snowflake Python)**: Calls Mistral API to generate metadata.  
- **Softmax Ranking**: Used for query relevance scoring.  

---

## **3. Snowflake Schema**  

### **3.1 Database Table**
```sql
CREATE TABLE DOCUMENTS (
    ID STRING PRIMARY KEY, 
    DOC_NAME STRING, 
    PARAGRAPH_NUMBER INT, 
    PARAGRAPH_TEXT STRING, 
    SIMPLE_QUESTIONS ARRAY, 
    COMPLEX_QUESTIONS ARRAY, 
    CONTEXT STRING, 
    SCOPE STRING
);
```

---

## **4. Snowflake UDF for Mistral API Call**  
```sql
CREATE OR REPLACE FUNCTION GENERATE_METADATA(paragraph STRING)
RETURNS OBJECT
LANGUAGE PYTHON
RUNTIME_VERSION = '3.8'
HANDLER = 'generate_metadata'
AS $$
import requests
import os

MISTRAL_API_KEY = os.getenv("MISTRAL_API_KEY")
MISTRAL_URL = "https://api.mistral.ai/v1/models/mistral-small-2409/completions"

def generate_metadata(paragraph):
    prompt = f"Extract metadata:\nParagraph: {paragraph}\n\nGenerate:\n- Simple Questions\n- Complex Questions\n- Context\n- Scope"
    headers = {"Authorization": f"Bearer {MISTRAL_API_KEY}", "Content-Type": "application/json"}
    payload = {"prompt": prompt, "max_tokens": 200}

    response = requests.post(MISTRAL_URL, json=payload, headers=headers)
    metadata = response.json()

    return {
        "simple_questions": metadata.get("simple_questions", []),
        "complex_questions": metadata.get("complex_questions", []),
        "context": metadata.get("context", ""),
        "scope": metadata.get("scope", "")
    }
$$;
```

---

## **5. Flask Application**  
### **5.1 `app.py`**  
```python
from flask import Flask, request, jsonify
from config import Config
from models import db
from services.document import ingest_document, query_document

app = Flask(__name__)
app.config.from_object(Config)
db.init_app(app)

@app.route('/api/v1/init', methods=['POST'])
def init_db():
    db.create_all()
    return jsonify({"message": "Database initialized successfully."}), 200

@app.route('/api/v1/ingest', methods=['POST'])
def ingest():
    data = request.json
    return ingest_document(data)

@app.route('/api/v1/query', methods=['GET'])
def query():
    query_text = request.args.get('query')
    return query_document(query_text)

if __name__ == '__main__':
    app.run(debug=True)
```

---

## **6. Configuration**  
### **6.1 `config.py`**  
```python
import os

class Config:
    SNOWFLAKE_USER = os.getenv('SNOWFLAKE_USER')
    SNOWFLAKE_PASSWORD = os.getenv('SNOWFLAKE_PASSWORD')
    SNOWFLAKE_ACCOUNT = os.getenv('SNOWFLAKE_ACCOUNT')
    SNOWFLAKE_DATABASE = os.getenv('SNOWFLAKE_DATABASE')
    SNOWFLAKE_SCHEMA = os.getenv('SNOWFLAKE_SCHEMA')
    MISTRAL_API_KEY = os.getenv('MISTRAL_API_KEY')
```

### **6.2 `.env`**
```
SNOWFLAKE_USER=your_user
SNOWFLAKE_PASSWORD=your_password
SNOWFLAKE_ACCOUNT=your_account
SNOWFLAKE_DATABASE=your_database
SNOWFLAKE_SCHEMA=your_schema
MISTRAL_API_KEY=your_mistral_key
```

---

## **7. Database Models**  
### **7.1 `models.py`**  
```python
from flask_sqlalchemy import SQLAlchemy

db = SQLAlchemy()

class Document(db.Model):
    __tablename__ = 'DOCUMENTS'
    id = db.Column(db.String, primary_key=True)
    doc_name = db.Column(db.String)
    paragraph_number = db.Column(db.Integer)
    paragraph_text = db.Column(db.Text)
    simple_questions = db.Column(db.ARRAY(db.String))
    complex_questions = db.Column(db.ARRAY(db.String))
    context = db.Column(db.Text)
    scope = db.Column(db.Text)
```

---

## **8. Document Processing Logic**  
### **8.1 `services/document.py`**  
```python
import uuid
import snowflake.connector
import os

def get_snowflake_connection():
    return snowflake.connector.connect(
        user=os.getenv('SNOWFLAKE_USER'),
        password=os.getenv('SNOWFLAKE_PASSWORD'),
        account=os.getenv('SNOWFLAKE_ACCOUNT'),
        database=os.getenv('SNOWFLAKE_DATABASE'),
        schema=os.getenv('SNOWFLAKE_SCHEMA')
    )

def ingest_document(data):
    doc_name = data['doc_name']
    paragraphs = data['paragraphs']

    conn = get_snowflake_connection()
    cursor = conn.cursor()

    for idx, paragraph in enumerate(paragraphs):
        metadata_query = "SELECT GENERATE_METADATA(%s)"
        cursor.execute(metadata_query, (paragraph,))
        metadata = cursor.fetchone()[0]

        insert_query = """
        INSERT INTO DOCUMENTS (ID, DOC_NAME, PARAGRAPH_NUMBER, PARAGRAPH_TEXT, SIMPLE_QUESTIONS, COMPLEX_QUESTIONS, CONTEXT, SCOPE)
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
        """
        cursor.execute(insert_query, (
            str(uuid.uuid4()), doc_name, idx, paragraph,
            metadata['simple_questions'], metadata['complex_questions'],
            metadata['context'], metadata['scope']
        ))

    conn.commit()
    cursor.close()
    conn.close()

    return {"message": "Document ingested successfully"}, 201

def query_document(query_text):
    conn = get_snowflake_connection()
    cursor = conn.cursor()

    # Softmax-based ranking query
    softmax_query = """
    SELECT PARAGRAPH_TEXT, CONTEXT, EXP(SIMILARITY_SCORE) / SUM(EXP(SIMILARITY_SCORE)) OVER () AS RANK
    FROM (
        SELECT PARAGRAPH_TEXT, CONTEXT, 
               COSINE_SIMILARITY(PARAGRAPH_TEXT, %s) AS SIMILARITY_SCORE
        FROM DOCUMENTS
    ) ORDER BY RANK DESC LIMIT 5
    """
    cursor.execute(softmax_query, (query_text,))
    results = cursor.fetchall()

    cursor.close()
    conn.close()

    return jsonify({"results": [{"paragraph": r[0], "context": r[1], "score": r[2]} for r in results]})
```

---

## **9. Execution Flow**
1. **Initialization (`/api/v1/init`)**  
   - Creates the Snowflake table if it doesn't exist.  

2. **Ingestion (`/api/v1/ingest`)**  
   - Splits document into paragraphs.  
   - Calls the Snowflake UDF to generate metadata using Mistral.  
   - Stores the paragraph and metadata in Snowflake.  

3. **Query (`/api/v1/query?query=...`)**  
   - Retrieves relevant paragraphs from Snowflake.  
   - Uses a **Softmax-based ranking** for optimal paragraph selection.  

---

## **10. Summary**
This system integrates **Snowflake** for structured storage, **Mistral-Small-2409** for metadata generation, and **Softmax ranking** for relevance-based retrieval. The architecture ensures efficient ingestion, metadata enrichment, and intelligent querying.
