# Postmaster
Postmaster is a composite vector retrieval system to bring efficiency and precision to knowledge retrieval.


# **Composite Vector-Based Retrieval System (CVRS)**
This document describes the **endpoints, ingestion process, vectorization strategy, and querying logic** using **multiple embeddings** (paragraph, simple questions, complex questions, context, scope) for precise information retrieval.

---

## **1️⃣ API Endpoints**  
| Endpoint        | Method | Description |
|---------------|--------|-------------|
| `/api/v1/init`  | POST  | Initializes the database for a tenant. |
| `/api/v1/ingest` | POST  | Ingests a document, extracts metadata, and stores vector embeddings. |
| `/api/v1/query`  | GET   | Searches for relevant content and retrieves top **n** paragraphs using softmax. |

---

## **2️⃣ Ingestion Process** (`/api/v1/ingest`)  
### **Step 1: User Uploads a Document**  
The user provides:  
- **Document URL** (PDF, DOC, TXT, etc.)  
- **Vector DB** (Pinecone, Chroma, FAISS, PGVector)  
- **Encoding Model API Key & URL**  
- **LLM API Key & Model**

Ah, I see! You want the payload for the **POST request** to be **separate** from the headers and just want to focus on the body of the request. Here's how you can send the document as an attachment in the **multipart/form-data** request payload.

### **POST Payload for Document Ingestion:**

Here’s what the **payload** would look like when sending the document as an attachment:

```json
{
  "tenant_id": "abc123",
  "vector_db": {
    "type": "pinecone",
    "url": "https://pinecone.io",
    "api_key": "pinecone-key"
  },
  "encoding_model": {
    "model": "bge-m3",
    "url": "https://encoder.com",
    "api_key": "encoder-key"
  },
  "llm_model": {
    "model": "gpt-4",
    "url": "https://llm.com",
    "api_key": "llm-api-key"
  },
  "doc_type": "PDF",
  "document": "<binary PDF content>"
}
```

### **Explanation of Payload Fields:**

1. **tenant_id**: A unique identifier for the tenant (user).
2. **vector_db**: The metadata required to access the vector DB (e.g., Pinecone).
   - **type**: The type of the vector DB (Pinecone, Chroma, FAISS, PGVector).
   - **url**: URL to the vector DB.
   - **api_key**: API key for authentication to the vector DB.
3. **encoding_model**: The metadata for the encoding model used to generate embeddings.
   - **model**: The model name for the encoder.
   - **url**: URL to the encoder model API.
   - **api_key**: API key for authentication to the encoder.
4. **llm_model**: The metadata for the large language model.
   - **model**: The model name (e.g., GPT-4).
   - **url**: URL to the LLM API.
   - **api_key**: API key for authentication to the LLM.
5. **doc_type**: The type of document being uploaded (e.g., "PDF").
6. **document**: This is the actual document being uploaded. You would typically send this as **binary data** as part of the `multipart/form-data` payload.

### **Multipart/form-data Request Example:**

In a **multipart/form-data** request, the **document** is the file that is uploaded along with other fields.

```http
POST /api/v1/ingest HTTP/1.1
Host: example.com
Content-Type: multipart/form-data; boundary=---Boundary123

---Boundary123
Content-Disposition: form-data; name="tenant_id"

abc123
---Boundary123
Content-Disposition: form-data; name="vector_db[type]"

pinecone
---Boundary123
Content-Disposition: form-data; name="vector_db[url]"

https://pinecone.io
---Boundary123
Content-Disposition: form-data; name="vector_db[api_key]"

pinecone-key
---Boundary123
Content-Disposition: form-data; name="encoding_model[model]"

bge-m3
---Boundary123
Content-Disposition: form-data; name="encoding_model[url]"

https://encoder.com
---Boundary123
Content-Disposition: form-data; name="encoding_model[api_key]"

encoder-key
---Boundary123
Content-Disposition: form-data; name="llm_model[model]"

gpt-4
---Boundary123
Content-Disposition: form-data; name="llm_model[url]"

https://llm.com
---Boundary123
Content-Disposition: form-data; name="llm_model[api_key]"

llm-api-key
---Boundary123
Content-Disposition: form-data; name="doc_type"

PDF
---Boundary123
Content-Disposition: form-data; name="document"; filename="research.pdf"
Content-Type: application/pdf

<binary file content here>
---Boundary123--
```

### **Key Parts of the Request:**

- **`Content-Disposition`**: Used to specify how the parts are handled. For the file part, it specifies `filename="research.pdf"`, which indicates that the uploaded file is named "research.pdf".
- **`Content-Type`**: For the file part, the `Content-Type` is `application/pdf` to indicate that the uploaded file is a PDF.
- **The file content**: The binary data of the PDF document is sent where `<binary file content here>` is indicated.

---
### **Step 2: Backend Processing**  
1. **Load and Parse the Document**  
   - Extract text, remove noise, chunk into **paragraphs**.  
   - If a **paragraph is too large**, return an error.

2. **Generate Metadata using LLM**  
   - For each paragraph, call LLM to generate:  
     ✅ **Simple Questions**  
     ✅ **Complex Questions**  
     ✅ **Context**  
     ✅ **Scope**  

**Example LLM Response:**  
```json
{
    "paragraph": "Quantum mechanics describes the behavior of subatomic particles...",
    "simple_questions": ["What is quantum mechanics?", "What are subatomic particles?"],
    "complex_questions": ["How does quantum mechanics relate to general relativity?"],
    "context": "Physics, Quantum Mechanics, Subatomic Particles",
    "scope": "Introduction to fundamental physics concepts."
}
```

---

### **Step 3: Generate and Store 5 Embeddings**  
Each **paragraph** and its **metadata** are converted into **vector embeddings** and stored in the **Vector DB**.

```python
def store_embeddings(paragraph, metadata):
    for key, text in metadata.items():
        embedding = encode_text(text)
        vector_db.insert(embedding, metadata={"type": key, "text": text})

store_embeddings(paragraph, {
    "paragraph": paragraph_text,
    "simple_questions": simple_questions,
    "complex_questions": complex_questions,
    "context": context,
    "scope": scope
})
```

---

## **3️⃣ Query Process (`/api/v1/query`)**  
### **Step 1: User Sends a Query**  
User queries the system using `/api/v1/query?query="How does quantum mechanics relate to general relativity?"`

---

### **Step 2: Convert Query to Embedding**  
- The query is **converted into a vector embedding** using the **same encoding model**.

```json
{
    "model": "bge-m3",
    "api_key": "encoder-key",
    "input": ["How does quantum mechanics relate to general relativity?"]
}
```

---

### **Step 3: Search Across All 5 Embeddings**  
- Search the **Vector DB** for the **top n** most similar results.  
- The system retrieves **paragraphs** and their **associated metadata**.

```python
query_embedding = get_query_embedding(user_query)
results = vector_db.query(vector=query_embedding, top_k=n, include_metadata=True)
```

---

### **Step 4: Apply Softmax to Rank Results**  
- Convert **similarity scores** into **probabilities** using the **softmax function**.

### Softmax Formula

![Softmax Formula](https://latex.codecogs.com/png.latex?S_i%20%3D%20%5Cfrac%7Be%5E%7Bx_i%7D%7D%7B%5Csum_%7Bj%3D1%7D%5En%20e%5E%7Bx_j%7D%7D)


##### **Example Softmax Calculation:**
```python
import numpy as np

def softmax(scores):
    exp_scores = np.exp(scores)
    return exp_scores / np.sum(exp_scores)

# Example similarity scores
similarity_scores = [0.9, 0.8, 0.85, 0.75, 0.7]
final_scores = softmax(similarity_scores)
```

---

### **Step 5: Select the Top n Paragraphs**  
- The **top n** paragraphs are selected **based on softmax ranking**.

```python
sorted_paragraphs = sorted(results, key=lambda x: x['softmax_score'], reverse=True)[:n]
selected_paragraphs = [p["text"] for p in sorted_paragraphs]
```

---

### **Step 6: Send Query + Context to LLM**  
- The selected **paragraphs** are sent **as context** to the LLM **along with the user query**.

**Example API Request to LLM:**  
```json
{
    "model": "gpt-4",
    "api_key": "llm-api-key",
    "context": [
        "Paragraph 1: Quantum mechanics describes subatomic particles...",
        "Paragraph 2: General relativity explains gravitational effects...",
        "Paragraph 3: The incompatibility of quantum mechanics and relativity..."
    ],
    "query": "How does quantum mechanics relate to general relativity?"
}
```

---

### **Step 7: LLM Generates the Final Answer**  
The LLM uses the **query + retrieved context** to generate the **final response**.

**Example LLM Response:**  
```json
{
    "response": "Quantum mechanics and general relativity describe different aspects of the universe. Quantum mechanics explains the behavior of subatomic particles, while general relativity describes the effects of gravity on a large scale. Scientists are working on theories like quantum gravity to bridge the gap."
}
```

---

## **4️⃣ Final Summary of Workflow**  

### **Ingestion (`/api/v1/ingest`)**
✅ Parse **document** and extract **paragraphs**.  
✅ Generate **Simple Questions, Complex Questions, Context, Scope** using **LLM**.  
✅ Convert **all 5 components** into **embeddings**.  
✅ Store **embeddings + metadata** in the **Vector DB**.  

### **Query (`/api/v1/query`)**
✅ Convert **user query** into an **embedding**.  
✅ Search **Vector DB** for **top n matches** across all **5 embedding types**.  
✅ Apply **softmax scoring** to rank the results.  
✅ Select **top n paragraphs** as **context**.  
✅ Send **query + context** to the **LLM**.  
✅ Return **final response** to the user.  
---

## 5️⃣ Advantages of This Approach

- Multi-Faceted Retrieval – Searches across paragraphs + metadata, not just raw text.
- Softmax-Based Ranking – Ensures the most relevant context is selected.
- Efficient LLM Usage – Reduces token costs by sending only top-ranked paragraphs.
- Improved Accuracy – Provides the LLM with better context, leading to better answers.
  
---

## 6️⃣ Pitfalls of the Six Methods & How CVRS Fixes Them

### 1. **Hierarchical Chunking**

**Pitfalls:**
- **Loss of Granularity**: While hierarchical chunking organizes content into nested layers, it may lose some granularity at lower levels, which can be important for nuanced context.
- **Over-Simplification**: The method might oversimplify complex content by collapsing too much information into broader chunks, potentially overlooking important details.

**How CVRS Fixes It:**
- **Fine-Grained Embeddings**: CVRS retrieves **embeddings** for **multiple components**: paragraphs, simple and complex questions, context, and scope. This ensures that even the smallest context is preserved and adequately represented.
- **Layered Retrieval**: Instead of collapsing information into larger chunks, CVRS evaluates the content across multiple layers, preserving more information and improving accuracy by considering a richer set of contexts.

---

### 2. **Sliding Window Chunking**

**Pitfalls:**
- **Context Loss at Edges**: In sliding window chunking, the content is divided into overlapping windows. However, chunks at the edges of windows can lose context as the boundaries are forced, which may reduce the quality of the retrieval.
- **Inconsistent Chunking**: The sliding window approach can sometimes create inconsistent chunks, where essential context might spill over or be fragmented.

**How CVRS Fixes It:**
- **Contextual Embedding**: By generating embeddings for **whole paragraphs** and associated components (simple/complex questions, context, and scope), CVRS avoids the rigid boundaries that can lead to context loss. It ensures that the full **semantic meaning** of the chunk is preserved, even across boundaries.
- **Flexible Retrieval**: CVRS uses a flexible multi-component retrieval system, considering the entire document context rather than just chunked windows, allowing for better coherence and consistency in the context of a query.

---

### 3. **Hypothetical Questions**

**Pitfalls:**
- **Misleading Results**: Hypothetical questions can generate answers that are not necessarily grounded in the real content, especially if the user’s query introduces uncertainty or ambiguity.
- **Over-Speculation**: Relying on hypothetical questions can lead to speculative answers, which may not align with the actual information in the content.

**How CVRS Fixes It:**
- **Grounded Context**: CVRS focuses on extracting **concrete context** by embedding **real content** into vector spaces and ensuring that results are based on **semantic similarity** with the original document, not hypothetical scenarios.
- **Rich Context and Scope**: By using context and scope embeddings, CVRS can ensure that answers are relevant to the user query without indulging in unnecessary speculation.

---

### 4. **Contextual Retrieval**

**Pitfalls:**
- **Limited by Query Understanding**: Traditional contextual retrieval systems may still rely heavily on keyword-based search, which limits their understanding of context and can return irrelevant results.
- **Context Narrowing**: Some methods focus too much on specific contexts, which can reduce the diversity of relevant information returned.

**How CVRS Fixes It:**
- **Multi-Layered Contextualization**: CVRS takes **multiple levels of context** into account: paragraphs, simple/complex questions, scope, and overall context. This allows the system to generate a **richer understanding** of the query and retrieve more relevant content based on a variety of semantic cues.
- **Diverse Contexts**: The system ensures that diverse contexts (like **questions and scope**) are retrieved and factored into the final result, preventing the narrowing down of results to irrelevant or overly specific contexts.

---

### 5. **Semantic Chunking**

**Pitfalls:**
- **Fragmentation of Meaning**: When chunking content semantically, there can still be instances where key relationships between different chunks are lost or incorrectly assigned, potentially leading to fragmented meanings.
- **Limited Handling of Complex Queries**: If chunks are not well-defined semantically, handling complex queries that span multiple chunks can be difficult and result in poor retrieval.

**How CVRS Fixes It:**
- **Embedding Layers for Semantics**: CVRS addresses this by generating **vector embeddings** for each content chunk and then combining them with other layers of meaning (such as context and scope). This ensures that even fragmented chunks retain their overall **semantic integrity**.
- **Holistic Retrieval**: CVRS goes beyond semantic chunking by considering the **full document context**, ensuring that complex queries which span multiple chunks are handled more effectively.

---

### 6. **HYDE (Hypothetical & Dynamic Embedding)**

**Pitfalls:**
- **Overfitting**: HYDE-based approaches can overfit to specific types of queries or content, limiting their ability to generalize across a broader range of topics.
- **Ambiguous Results**: Because HYDE focuses on hypothetical dynamic embeddings, it can generate **ambiguous results** when the query is unclear or when content isn’t well defined within the system.

**How CVRS Fixes It:**
- **Rich Vector-Based Approach**: Unlike HYDE’s reliance on dynamic embeddings, CVRS uses **static and dynamic embeddings** from multiple **contextual layers**, including **paragraphs, questions, and scope**, allowing the system to better handle **diverse queries** without overfitting.
- **Clearer Results**: By using embeddings for real content and ensuring that the **semantic meaning** is well-preserved, CVRS produces results grounded in the actual content, reducing ambiguity.

---

## 7️⃣ Conclusion

While methods like **hierarchical chunking**, **sliding window chunking**, **hypothetical questions**, **contextual retrieval**, **semantic chunking**, and **HYDE** have their merits, they each face limitations, particularly when it comes to **context preservation**, **handling complex queries**, and **semantic ambiguity**. The **Composite Vector-Based Retrieval System (CVRS)** addresses these issues by combining **multi-layer embeddings** that represent different aspects of content, from **paragraphs** to **simple and complex questions**, **context**, and **scope**. CVRS ensures more **accurate**, **flexible**, and **contextually aware** retrieval, improving both **precision** and **relevance** for user queries.
