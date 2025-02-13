import fitz  # PyMuPDF for PDF extraction
import uuid
from flask import jsonify
from models import SnowflakeDB

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

def ingest_document(data):
    """Handle document ingestion and store in Snowflake."""
    if "filename" not in data or "content" not in data:
        return jsonify({"error": "Missing filename or content"}), 400

    db = SnowflakeDB()
    document_id = str(uuid.uuid4())
    filename = data["filename"]
    content = data["content"]

    # Store document metadata in Snowflake
    db.execute_query(
        "INSERT INTO DOCUMENTS (ID, FILENAME, CONTENT) VALUES (%s, %s, %s)",
        (document_id, filename, content)
    )

    # Extract paragraphs
    paragraphs = extract_paragraphs_from_pdf(content)

    # Process paragraphs and store analysis in Snowflake
    for para in paragraphs:
        paragraph_id = str(uuid.uuid4())

        # Call Snowflake UDF for text analysis
        result = db.execute_query("SELECT GENERATE_QUESTIONS(%s)", (para,))
        qa_data = result[0][0]  # JSON object returned from UDF

        db.execute_query(
            """INSERT INTO PARAGRAPH_ANALYSIS (ID, DOCUMENT_ID, PARAGRAPH, SIMPLE_QUESTIONS, COMPLEX_QUESTIONS, CONTEXT, SCOPE) 
               VALUES (%s, %s, %s, %s, %s, %s, %s)""",
            (
                paragraph_id,
                document_id,
                para,
                qa_data.get("Simple Questions", ""),
                qa_data.get("Complex Questions", ""),
                qa_data.get("Context", ""),
                qa_data.get("Scope", ""),
            )
        )

    db.commit()
    db.close()

    return jsonify({"message": "Document ingested successfully", "paragraphs": len(paragraphs)})

def query_document(query_text):
    """Retrieve paragraphs and analysis from Snowflake."""
    db = SnowflakeDB()
    results = db.execute_query(
        "SELECT PARAGRAPH, SIMPLE_QUESTIONS, COMPLEX_QUESTIONS, CONTEXT, SCOPE FROM PARAGRAPH_ANALYSIS WHERE PARAGRAPH ILIKE %s",
        (f"%{query_text}%",)
    )

    response = [
        {
            "paragraph": row[0],
            "simple_questions": row[1],
            "complex_questions": row[2],
            "context": row[3],
            "scope": row[4]
        } for row in results
    ]

    db.close()
    return jsonify(response)
