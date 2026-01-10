from mcp.server.fastmcp import FastMCP 
from dotenv import load_dotenv 
import httpx
import json
import os 
from bs4 import BeautifulSoup

load_dotenv()

mcp = FastMCP("docs") 

USER_AGENT = "docs-app/1.0"
SERPER_URL = "https://google.serper.dev/search"

docs_urls = {
    "langchain" : "python.langchain.com/docs", 
    "llama-index" : "docs.llamaindex.ai/en/stable", 
    "openai" : "platform.openai.com/docs", 
}

# helper functions: 
async def search_web(query: str) -> dict | None:
    payload = json.dumps({"q" : query, "num" : 2}) # check only top 2 

    headers = {
        "X-API-KEY" : os.getenv("SERPER_API_KEY"), # API Key 
        "Content-Type" : "application/json", 
    }

    async with httpx.AsyncClient() as client: 
        try: 
            response = await client.post(
                SERPER_URL, headers=headers, data=payload, timeout=30.0
            )
            response.raise_for_status() 
            return response.json() 
        except httpx.TimeoutException:
            return {"organic" : []} 
        

async def fetch_url(url: str):
    async with httpx.AsyncClient() as client: 
        try: 
            response = await client.get(
                SERPER_URL, timeout=30.0
            )
            soup = BeautifulSoup(response.text, "html.parser")
            text = soup.get_text()
            return text
        except httpx.TimeoutException:
            return "Timeout error"

@mcp.tool()
async def get_docs(query: str, library: str):
    """ 
    // clear descriptoin that make the llm agent to understand what this tool is about. 
    // so, they can use this tool in 
    Search the docs for a given query and library. 
    Supports langchain, openai, and llma-index. 

    Args: 
        query: The query to search for (e.g. "Chroma DB") 
        library: The library to search in (e.g. "langchain") 
    
    Returns:
        Text from the docs 
    """
    if library not in docs_urls: 
        raise ValueError(f"Library {library} not supported by this tool") 
    
    query = f"site:{docs_urls[library]} {query}"
    results = await search_web(query)
    if len(results["organic"]) == 0:
        return "No results found"
    
    text = ""
    for result in results["organic"]:
        text += await fetch_url(result["link"])
    return text

# serper API 

if __name__ == "__main__":
    mcp.run(transport="stdio")
