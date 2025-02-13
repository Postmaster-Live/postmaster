from flask import Flask, request, jsonify
from config import Config
from services.document import ingest_document, query_document
from models import SnowflakeDB

app = Flask(__name__)
app.config.from_object(Config)

@app.route('/api/v1/init', methods=['POST'])
def init_db():
    """Initialize Snowflake tables and UDFs."""
    db = SnowflakeDB()
    with open("snowflake_setup.sql", "r") as f:
        queries = f.read().split(";")
        for query in queries:
            if query.strip():
                db.execute_query(query)
    db.commit()
    db.close()
    return jsonify({"message": "Snowflake tables & UDFs initialized successfully."})

@app.route('/api/v1/ingest', methods=['POST'])
def ingest():
    return ingest_document(request.json)

@app.route('/api/v1/query', methods=['GET'])
def query():
    return query_document(request.args.get('query'))

if __name__ == '__main__':
    app.run(debug=True)
