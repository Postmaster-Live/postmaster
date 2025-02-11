from flask import Flask, request, jsonify
from config import Config
from models import db
from services.document import ingest_document, query_document

app = Flask(__name__)
app.config.from_object(Config)
db.init_app(app)

# Route to initialize the database
@app.route('/api/v1/init', methods=['POST'])
def init_db():
    db.create_all()
    return jsonify({"message": "Database initialized successfully."}), 200

# Route to ingest a document
@app.route('/api/v1/ingest', methods=['POST'])
def ingest():
    data = request.json
    return ingest_document(data)

# Route to query documents
@app.route('/api/v1/query', methods=['GET'])
def query():
    query_text = request.args.get('query')
    return query_document(query_text)

if __name__ == '__main__':
    app.run(debug=True)
