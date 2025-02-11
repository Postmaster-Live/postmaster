# **Postmaster**

## **Limitations of Traditional RAG Approaches**

Traditional Retrieval-Augmented Generation (RAG) systems primarily rely on single-vector embeddings for knowledge retrieval, which can limit efficiency and precision. These approaches typically use a single-vector representation per document, leading to several challenges:

1. **Loss of Context** – A single embedding struggles to capture multiple facets of a document, leading to ambiguous or incomplete retrieval.
2. **Shallow Relevance Matching** – Keyword-based vector search often prioritizes surface-level similarity rather than capturing deeper semantic meaning.
3. **Limited Adaptability** – Traditional RAG systems lack the ability to dynamically adjust search granularity based on the complexity of the query.

**Postmaster** addresses these challenges by introducing **composite embeddings**, which provide richer, multi-perspective representations and enable more context-aware search capabilities.

---
### project Structure**

```
/
│
├── app.py            # Main entry point for the app
├── config.py         # Configuration settings
├── models.py         # Database models (with pgvector)
├── services/         # Folder for business logic
│   ├── embedding.py  # Logic for generating embeddings
│   ├── document.py   # Logic for handling documents (ingestion and queries)
│
├── requirements.txt  # Python dependencies
└── .env              # Environment variables
```

### **How to Run the Application**

1. **Set up your PostgreSQL database**:
   - Ensure that the PostgreSQL database is running and that the `pgvector` extension is installed.
   
   ```sql
   CREATE EXTENSION IF NOT EXISTS vector;
   ```

2. **Set environment variables** for your database URL and secret key in `.env` or in your terminal:

   ```
   export DATABASE_URL=postgresql://username:password@localhost:5432/database_name
   export SECRET_KEY=your_secret_key
   ```

3. **Install dependencies**:
   Run the following command to install the required dependencies:

   ```bash
   pip install -r requirements.txt
   ```

4. **Run the Flask app**:
   Start the Flask application with:

   ```bash
   python app.py
   ```

---
