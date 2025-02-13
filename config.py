import os
from dotenv import load_dotenv

# Load .env variables
load_dotenv()

class Config:
    # Snowflake
    SNOWFLAKE_USER = os.getenv("SNOWFLAKE_USER")
    SNOWFLAKE_PASSWORD = os.getenv("SNOWFLAKE_PASSWORD")
    SNOWFLAKE_ACCOUNT = os.getenv("SNOWFLAKE_ACCOUNT")
    SNOWFLAKE_DATABASE = os.getenv("SNOWFLAKE_DATABASE")
    SNOWFLAKE_SCHEMA = os.getenv("SNOWFLAKE_SCHEMA")
    SNOWFLAKE_WAREHOUSE = os.getenv("SNOWFLAKE_WAREHOUSE")
    SNOWFLAKE_ROLE = os.getenv("SNOWFLAKE_ROLE")

    # Mistral API Key
    MISTRAL_API_KEY = os.getenv("MISTRAL_API_KEY")
