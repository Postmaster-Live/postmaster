from flask import Flask, request, jsonify
from services.document import ingest_document, query_document
from models import SnowflakeDB

app = Flask(__name__)

@app.route('/init', methods=['POST'])
def init_db():
    db = SnowflakeDB()
    with open("snowflake_setup.sql", "r") as f:
        for query in f.read().split(";"):
            if query.strip():
                db.execute_query(query)
    db.commit()
    db.close()
    return jsonify({"message": "Snowflake initialized."})

@app.route('/ingest', methods=['POST'])
def ingest():
    return ingest_document(request.json)

@app.route('/query', methods=['GET'])
def query():
    return query_document(request.args.get('query'))

if __name__ == '__main__':
    app.run(debug=True)
