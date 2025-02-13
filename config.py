import os

class Config:
    # Snowflake Configuration
    SNOWFLAKE_USER = os.getenv("SNOWFLAKE_USER", "your_user")
    SNOWFLAKE_PASSWORD = os.getenv("SNOWFLAKE_PASSWORD", "your_password")
    SNOWFLAKE_ACCOUNT = os.getenv("SNOWFLAKE_ACCOUNT", "your_account")
    SNOWFLAKE_DATABASE = os.getenv("SNOWFLAKE_DATABASE", "your_database")
    SNOWFLAKE_SCHEMA = os.getenv("SNOWFLAKE_SCHEMA", "your_schema")
    SNOWFLAKE_WAREHOUSE = os.getenv("SNOWFLAKE_WAREHOUSE", "your_warehouse")
    SNOWFLAKE_ROLE = os.getenv("SNOWFLAKE_ROLE", "your_role")
    
    # OpenAI API Key
    OPENAI_API_KEY = os.getenv("OPENAI_API_KEY", "your_openai_key")
