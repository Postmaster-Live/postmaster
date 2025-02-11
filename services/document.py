from models import db, Document
from .embedding import generate_embedding
from flask import jsonify

# Ingest document and store it in the database
def ingest_document(data):
    tenant_id = data.get('tenant_id')
    doc_type = data.get('doc_type')
    content = data.get('document')

    # Generate the vector representation for the document content
    vector = generate_embedding(content)

    # Create a new document record
    new_document = Document(tenant_id=tenant_id, doc_type=doc_type, content=content, vector=vector)
    db.session.add(new_document)
    db.session.commit()

    return {"message": "Document ingested successfully."}, 200

# Query the database for similar documents based on a given query
def query_document(query_text):
    # Generate vector for the query
    query_vector = generate_embedding(query_text)

    # Perform similarity search in the database using pgvector's cosine similarity
    results = Document.query.filter(
        Document.vector.cosine_distance(query_vector) < 0.1
    ).all()

    documents = [{"id": doc.id, "content": doc.content} for doc in results]
    return {"results": documents}, 200
