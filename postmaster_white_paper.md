# **Postmaster**

Postmaster leverages a composite vector database to bring efficiency and precision to knowledge retrieval.

## **Limitations of Traditional RAG Approaches**

Traditional Retrieval-Augmented Generation (RAG) systems primarily rely on single-vector embeddings for knowledge retrieval, which can limit efficiency and precision. These approaches typically use a single-vector representation per document, leading to several challenges:

1. **Loss of Context** – A single embedding struggles to capture multiple facets of a document, leading to ambiguous or incomplete retrieval.
2. **Shallow Relevance Matching** – Keyword-based vector search often prioritizes surface-level similarity rather than capturing deeper semantic meaning.
3. **Limited Adaptability** – Traditional RAG systems lack the ability to dynamically adjust search granularity based on the complexity of the query.

**Postmaster** addresses these challenges by introducing **composite embeddings**, which provide richer, multi-perspective representations and enable more context-aware search capabilities.

---

# **Composite Vector-Based Retrieval System (CVRS)**  
This document outlines the **endpoints, ingestion process, vectorization strategy, and querying logic** utilizing **composite embeddings** (paragraph, simple questions, complex questions, context, scope) to ensure precise information retrieval.


---

##  API Endpoints  
| Endpoint        | Method | Description |
|-----------------|--------|-------------|
| `/api/v1/init`  | POST   | Initializes the database for a tenant. |
| `/api/v1/ingest` | POST   | Ingests a document, extracts metadata, and stores vector embeddings. |
| `/api/v1/query`  | GET   | Searches for relevant content and retrieves the top **n** paragraphs using softmax-based ranking, which the LLM will use to generate the response. |

Authentication: All endpoints require an API key in the request header:Authorization: Bearer <your_api_key>


---


### **1️⃣ Initialization Method**: `/api/v1/init`

#### **Purpose**:
The `init` method initializes the necessary infrastructure and databases for a specific tenant.

#### **Request Method**:
- **POST**

#### **Endpoint**:
```
POST /api/v1/init
```

#### **Request Payload**:
```json
{
  "tenant_id": "abc123",
  "vector_db": {
    "type": "pinecone",
    "url": "https://pinecone.io",
    "api_key": "pinecone-key"
  }
}
```

#### **Request Parameters**:

1. **`tenant_id`**:
   - **Description**: A unique identifier for the tenant in the system.
   - **Type**: String
   - **Example**: `"abc123"`

2. **`vector_db`**:
   - **Description**: Specifies the vector database to be used for storing embeddings.
   - **Type**: Object
     - **`type`**: The type of vector database (e.g., Pinecone, Chroma).
     - **`url`**: The URL to access the vector database.
     - **`api_key`**: The API key required for authentication to the vector database.
   - **Example**:  
     ```json
     {
       "type": "pinecone",
       "url": "https://pinecone.io",
       "api_key": "pinecone-key"
     }
     ```

#### **Response**:
If successful, the response indicates that the tenant-specific configuration has been initialized.

**Success Response**:
```json
{
  "status": "success",
  "message": "Tenant 'abc123' initialized successfully."
}
```

#### **Failure Response**:
If there’s any issue with the provided data (e.g., invalid API keys, incorrect URLs), the response will detail the error.

**Error Response**:
```json
{
  "status": "error",
  "message": "Invalid API key for the vector database."
}
```

#### **Process**:
1. **Validate Input**: The system checks that all required fields are provided and valid, including the tenant ID and vector database configuration.
2. **Configure Vector Database**: The system sets up the chosen vector database (e.g., Pinecone, Chroma) and authenticates using the provided credentials.
3. **Return Success**: Once the initialization is complete, the system returns a success response.

#### **Usage Example**:

Let’s assume you want to initialize a tenant with `tenant_id="abc123"` and configure Pinecone as the vector database. You would send the following request:

```bash
curl -X POST "https://your-api-url.com/api/v1/init" \
-H "Authorization: Bearer your-api-key" \
-H "Content-Type: application/json" \
-d '{
  "tenant_id": "abc123",
  "vector_db": {
    "type": "pinecone",
    "url": "https://pinecone.io",
    "api_key": "pinecone-key"
  }
}'
```

----


## **2️⃣ Ingestion Process** (`/api/v1/ingest`)

### **Step 1: User Uploads a Document**  
The user provides:  
- **Document URL** (PDF, DOC, TXT, etc.)  
- **Vector DB** (Pinecone, Chroma, Snowflake, PGVector)  
- **Encoding Model API Key & URL**  
- **LLM API Key & Model**

### **POST Payload for Document Ingestion:**

Here’s an example of what the **payload** would look like when sending the document as an attachment:

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
2. **vector_db**: Metadata for accessing the vector database (e.g., Pinecone, Snowflake).
   - **type**: The type of vector DB (Pinecone, Chroma, Snowflake, PGVector).
   - **url**: URL to the vector DB.
   - **api_key**: API key for authentication to the vector DB.
3. **encoding_model**: Metadata for the encoding model used to generate embeddings.
   - **model**: The encoder model's name.
   - **url**: URL to the encoder model API.
   - **api_key**: API key for authentication to the encoder.
4. **llm_model**: Metadata for the large language model (LLM).
   - **model**: The model name (e.g., GPT-4).
   - **url**: URL to the LLM API.
   - **api_key**: API key for authentication to the LLM.
5. **doc_type**: The document type being uploaded (e.g., "PDF").
6. **document**: The actual document to be uploaded, typically sent as **binary data** within the `multipart/form-data` payload.

### **Multipart/form-data Request Example:**

In a **multipart/form-data** request, the **document** is uploaded along with other fields.

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

- **`Content-Disposition`**: Specifies how parts are handled. For the file part, it specifies `filename="research.pdf"`, indicating the file name.
- **`Content-Type`**: For the file part, `application/pdf` indicates the file type.
- **The file content**: The PDF binary data is sent where `<binary file content here>` is indicated.

---

### **Step 2: Backend Processing**  
1. **Load and Parse the Document**  
   - Extract text, remove noise, and chunk into **paragraphs**.  
   - If a **paragraph is too large**, an error is returned.

2. **Generate Metadata using LLM**  
   - For each paragraph, the LLM is called to generate:  
     ✅ **Simple Questions**  
     ✅ **Complex Questions**  
     ✅ **Context**  
     ✅ **Scope**  

## LLM Prompt:

You are an expert in [Topic]. Generate structured questions along with context and scope to help someone deeply understand the topic.  

### Guidelines:  
- **Context:** Provide a brief background on the topic.  
- **Scope:** Define the key areas covered within the topic.  
- **Simple Questions:** Focused on basic facts and definitions.  
- **Complex Questions:** Require critical thinking, analysis, or multi-step reasoning.  
- The number of questions should be determined based on the complexity and breadth of the topic.  

### Input:
- **Topic:** [Topic Name]  
- **Subtopics:** [List of relevant subtopics]  

### Output Format (JSON):  
```json
{
  "context": "[Provide a concise background on the topic, explaining its relevance and key concepts.]",
  "scope": "[Define the boundaries of the discussion—what aspects of the topic will be explored.]",
  "simple_questions": [
    "[Question 1]",
    "[Question 2]",
    "[... More as needed]"
  ],
  "complex_questions": [
    "[Question 1]",
    "[Question 2]",
    "[... More as needed]"
  ]
}
```

---

### Example LLM Response:  
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
The user queries the system using:  
`/api/v1/query?query="How does quantum mechanics relate to general relativity?"`

---

### **Step 2: Convert Query to Embedding**  
The query is **converted into a vector embedding** using the **same encoding model**.

```json
{
    "model": "bge-m3",
    "api_key": "encoder-key",
    "input": ["How does quantum mechanics relate to general relativity?"]
}
```

---

### **Step 3: Search Across All 5 Embeddings**  
The system searches the **Vector DB** for the **top n** most similar results.  
It retrieves **paragraphs** and their **associated metadata**.

```python
query_embedding = get_query_embedding(user_query)
results = vector_db.query(vector=query_embedding, top_k=n, include_metadata=True)
```

---

### **Step 4: Apply Softmax to Rank Results**  
The system converts the **similarity scores** into **probabilities** using the **softmax function**.

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
### Why is Softmax used instead of selecting the maximum score?

Softmax-based ranking is used instead of simply selecting the maximum score because it weights all results proportionally rather than relying on a single "best" match.

#### Relative Relevance:
Softmax converts raw similarity scores into probabilities, reflecting how much each result stands out relative to others. A paragraph with a slightly lower score might still be highly relevant if other options are weak, which maximum-score selection would miss.

#### Balanced Context:
By considering all results, softmax avoids over-relying on a single embedding type (e.g., a paragraph might score lower in "simple questions" but higher in "context" embeddings). This ensures a more holistic selection of context for the LLM.

#### Score Normalization:
Softmax accounts for differences in absolute score magnitudes. For example, a score of 0.9 in one query might be weaker than 0.8 in another, depending on the distribution of all scores. Softmax adjusts for this dynamically.

In short, Softmax ensures the system compares results contextually, not just absolutely, leading to better-ranked, more useful paragraphs for the LLM to generate answers.

---

### **Step 5: Select the Top n Paragraphs**  
The **top n** paragraphs are selected **based on softmax ranking**.

```python
sorted_paragraphs = sorted(results, key=lambda x: x['softmax_score'], reverse=True)[:n]
selected_paragraphs = [p["text"] for p in sorted_paragraphs]
```

---

### **Step 6: Send Query + Context to LLM**  
The selected **paragraphs** are sent **as context** to the LLM **along with the user query**.

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

### **Initialization (`/api/v1/init`)**
✅ Create Database  **database tables** and setup other things necessary for initializing the tenant. 

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
