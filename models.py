from flask_sqlalchemy import SQLAlchemy
from pgvector.sqlalchemy import Vector

db = SQLAlchemy()

class Document(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    tenant_id = db.Column(db.String(100), nullable=False)
    doc_type = db.Column(db.String(50), nullable=False)
    content = db.Column(db.Text, nullable=False)
    vector = db.Column(Vector(1536), nullable=False)  # Adjust dimension as per your model

    def __repr__(self):
        return f"<Document {self.id} - {self.tenant_id}>"
