import fitz  # PyMuPDF for PDF extraction
import uuid
import numpy as np
from flask import jsonify
from models import SnowflakeDB
import requests
from config import Config

def extract_paragraphs_from_pdf(file_path):
    """Extract paragraphs from a PDF file."""
    doc = fitz.open(file_path)
    paragraphs = []

    for page in doc:
        text = page.get_text("text")
        for para in text.split("\n\n"):
            if para.strip():
                paragraphs.append(para.strip())

    return paragraphs

def softmax(scores):
    """Apply softmax function to a list of scores."""
    exp_scores = np.exp(scores - np.max(scores))
    return exp_scores / exp_scores.sum()

def generate_questions_mistral(text):
    """Call Mistral API for question generation."""
    API_URL = "https://api.mistral.ai/v1/chat/completions"
    HEADERS = {
        "Authorization": f"Bearer {Config.MISTRAL_API_KEY}",
        "Content-Type": "application/json"
    }

    data = {
        "model": "mistral-small-2409",
        "messages": [{"role": "user", "content": f"Generate questions, context, and scope for: {text}"}],
        "temperature": 0.7
    }

    response = requests.post(API_URL, headers=HEADERS, json=data)
    return response.json()["choices"][0]["message"]["content"]

def ingest_document(data):
    """Handle document ingestion and store in Snowflake."""
    if "filename" not in data or "content" not in data:
        return jsonify({"error": "Missing filename or content"}), 400

    db = SnowflakeDB()
    document_id = str(uuid.uuid4())
    filename = data["filename"]
    content = data["content"]

    db.execute_query(
        "INSERT INTO DOCUMENTS (ID, FILENAME, CONTENT) VALUES (%s, %s, %s)",
        (document_id, filename, content)
    )

    paragraphs = extract_paragraphs_from_pdf(content)
    scores = np.random.rand(len(paragraphs))  
    softmax_scores = softmax(scores)

    for idx, para in enumerate(paragraphs):
        paragraph_id = str(uuid.uuid4())
        qa_data = generate_questions_mistral(para)

        db.execute_query(
            """INSERT INTO PARAGRAPH_ANALYSIS (ID, DOCUMENT_ID, PARAGRAPH, SIMPLE_QUESTIONS, COMPLEX_QUESTIONS, CONTEXT, SCOPE, SCORE) 
               VALUES (%s, %s, %s, %s, %s, %s, %s, %s)""",
            (paragraph_id, document_id, para, qa_data, "", "", "", softmax_scores[idx])
        )

    db.commit()
    db.close()

    return jsonify({"message": "Document ingested successfully", "paragraphs": len(paragraphs)})

def query_document(query_text):
    """Retrieve paragraphs ranked by Softmax."""
    db = SnowflakeDB()
    results = db.execute_query(
        """SELECT PARAGRAPH, SCORE FROM PARAGRAPH_ANALYSIS WHERE PARAGRAPH ILIKE %s ORDER BY SCORE DESC""",
        (f"%{query_text}%",)
    )

    response = [{"paragraph": row[0], "score": row[1]} for row in results]

    db.close()
    return jsonify(response)
