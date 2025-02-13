# **Postmaster**

## **Limitations of Traditional RAG Approaches**

Traditional Retrieval-Augmented Generation (RAG) systems primarily rely on single-vector embeddings for knowledge retrieval, which can limit efficiency and precision. These approaches typically use a single-vector representation per document, leading to several challenges:

1. **Loss of Context** – A single embedding struggles to capture multiple facets of a document, leading to ambiguous or incomplete retrieval.
2. **Shallow Relevance Matching** – Keyword-based vector search often prioritizes surface-level similarity rather than capturing deeper semantic meaning.
3. **Limited Adaptability** – Traditional RAG systems lack the ability to dynamically adjust search granularity based on the complexity of the query.

**Postmaster** addresses these challenges by introducing **composite embeddings**, which provide richer, multi-perspective representations and enable more context-aware search capabilities.

---
### Project Structure

```
/
│
├── app.py            # Main entry point for the app
├── config.py         # Configuration settings
├── models.py         # Database models (with pgvector)
├── services/         # Folder for business logic
│   ├── document.py   # Logic for handling documents (ingestion and queries)
│
├── requirements.txt  # Python dependencies
└── .env              # Environment variables
```

## ** System Architecture**  
- **Flask API**: Handles ingestion and querying of documents.  
- **Snowflake Database**: Stores extracted paragraphs and metadata.  
- **Mistral-Small-2409**: Generates metadata for each paragraph.  
- **UDF (Snowflake Python)**: Calls Mistral API to generate metadata.  
- **Softmax Ranking**: Used for query relevance scoring.


---

## ** Snowflake Database Table**
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

## ** Snowflake UDF for Mistral API Call**  
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


## ** `.env`**
```
SNOWFLAKE_USER=your_user
SNOWFLAKE_PASSWORD=your_password
SNOWFLAKE_ACCOUNT=your_account
SNOWFLAKE_DATABASE=your_database
SNOWFLAKE_SCHEMA=your_schema
MISTRAL_API_KEY=your_mistral_key
```

---


## **How to Run the Application**

   ```
1. **Install dependencies**:
   Run the following command to install the required dependencies:

   ```bash
   pip3 install -r requirements.txt
   ```

2. **Run the Flask app**:
   Start the Flask application with:

   ```bash
   python3 app.py
   ```

---

## ** Execution Flow**
1. **Initialization (`/api/v1/init`)**  
   - Creates the Snowflake table if it doesn't exist.  

2. **Ingestion (`/api/v1/ingest`)**  
   - Splits document into paragraphs.  
   - Calls the Snowflake UDF to generate metadata using Mistral.  
   - Stores the paragraph and metadata in Snowflake.  

3. **Query (`/api/v1/query?query=...`)**  
   - Retrieves relevant paragraphs from Snowflake.  
   - Uses a **Softmax-based ranking** for optimal paragraph selection.  


