#!/usr/bin/env python3
"""
Scraper - Los ojos de Eden
Extrae contenido web para conocimiento del organismo
Usa requests + BeautifulSoup para compatibilidad
"""

import re
import sqlite3
import sys
import time
from datetime import datetime
from typing import List, Tuple, Optional

try:
    import requests
    from bs4 import BeautifulSoup
    USE_BEAUTIFUL_SOUP = True
except ImportError:
    USE_BEAUTIFUL_SOUP = False

# Configuracion
DB_PATH = "/home/ubuntu/eden_kg.db"
MAX_CHARS = 2000
FETCH_TIMEOUT = 30  # segundos

# Fuentes a escanear
SOURCES = [
    {
        "url": "https://news.ycombinator.com",
        "name": "hacker_news",
        "tipo": "tendencias_tech",
        "schedule_hours": 6
    },
    {
        "url": "https://github.com/trending",
        "name": "github_trending",
        "tipo": "repos_trending",
        "schedule_hours": 6
    },
    {
        "url": "https://arxiv.org/list/cs.AI/recent",
        "name": "arxiv_ai",
        "tipo": "papers_ia",
        "schedule_hours": 24
    },
]


def init_web_knowledge_table(conn: sqlite3.Connection) -> None:
    """Inicializa tabla web_knowledge si no existe."""
    conn.execute("""
        CREATE TABLE IF NOT EXISTS web_knowledge (
            id TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            fuente TEXT NOT NULL,
            fuente_url TEXT NOT NULL,
            tipo TEXT NOT NULL,
            titulo TEXT,
            contenido TEXT NOT NULL,
            usado_por TEXT,
            created_at TEXT NOT NULL
        )
    """)
    conn.execute("""
        CREATE TABLE IF NOT EXISTS web_scan_schedule (
            fuente TEXT PRIMARY KEY,
            last_scan TEXT,
            next_scan TEXT,
            estado TEXT DEFAULT 'idle'
        )
    """)
    conn.commit()


def generate_id() -> str:
    """Genera ID unico basado en timestamp."""
    return format(int(time.time() * 1000), 'x')


def clean_text(html_content: str, max_chars: int = MAX_CHARS) -> str:
    """Limpia HTML y devuelve texto limpio."""
    if not html_content:
        return ""

    # Eliminar scripts y estilos
    text = re.sub(r'<script[^>]*>.*?</script>', '', html_content, flags=re.DOTALL | re.IGNORECASE)
    text = re.sub(r'<style[^>]*>.*?</style>', '', text, flags=re.DOTALL | re.IGNORECASE)
    text = re.sub(r'<noscript[^>]*>.*?</noscript>', '', text, flags=re.DOTALL | re.IGNORECASE)

    # Remove HTML tags
    text = re.sub(r'<[^>]+>', ' ', text)

    # Decode HTML entities
    text = text.replace('&nbsp;', ' ')
    text = text.replace('&lt;', '<')
    text = text.replace('&gt;', '>')
    text = text.replace('&amp;', '&')
    text = text.replace('&quot;', '"')
    text = text.replace('&#39;', "'")

    # Normalize whitespace
    text = re.sub(r'\s+', ' ', text)

    # Trim and limit
    text = text.strip()

    if len(text) > max_chars:
        text = text[:max_chars] + "..."

    return text


def extract_content(url: str) -> Optional[str]:
    """Extrae contenido limpio de una URL."""
    try:
        headers = {
            'User-Agent': 'Mozilla/5.0 (compatible; Eden/1.0; +http://eden.ai)'
        }
        response = requests.get(url, headers=headers, timeout=FETCH_TIMEOUT)

        if response.status_code != 200:
            print(f"[SCRAPER] HTTP {response.status_code} para {url}", file=sys.stderr)
            return None

        if USE_BEAUTIFUL_SOUP:
            soup = BeautifulSoup(response.text, 'html.parser')
            # Remove script and style elements
            for script in soup(["script", "style"]):
                script.decompose()
            content = soup.get_text(separator=' ', strip=True)
        else:
            content = clean_text(response.text)

        # Limit and clean
        if len(content) > MAX_CHARS:
            content = content[:MAX_CHARS] + "..."

        return content.strip()

    except Exception as e:
        print(f"[SCRAPER] Error extrayendo {url}: {e}", file=sys.stderr)
        return None


def save_entry(conn: sqlite3.Connection, fuente: str, url: str, tipo: str, titulo: str, contenido: str) -> None:
    """Guarda una entrada en web_knowledge."""
    now = datetime.utcnow().isoformat() + "Z"
    entry_id = generate_id()

    conn.execute("""
        INSERT INTO web_knowledge (id, timestamp, fuente, fuente_url, tipo, titulo, contenido, usado_por, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, (entry_id, now, fuente, url, tipo, titulo, contenido[:MAX_CHARS], None, now))
    conn.commit()


def extract_hacker_news(conn: sqlite3.Connection) -> int:
    """Extrae tendencias de Hacker News."""
    print("[SCRAPER] Escaneando Hacker News...")
    url = "https://news.ycombinator.com"
    content = extract_content(url)

    if not content:
        print(f"[SCRAPER] Fallo extraccion de {url}")
        return 0

    save_entry(conn, "hacker_news", url, "tendencias_tech", "Tendencias Tech HN", content)
    print(f"[SCRAPER] HN: guardado {len(content)} chars")
    return 1


def extract_github_trending(conn: sqlite3.Connection) -> int:
    """Extrae repos trending de GitHub."""
    print("[SCRAPER] Escaneando GitHub Trending...")
    url = "https://github.com/trending"
    content = extract_content(url)

    if not content:
        print(f"[SCRAPER] Fallo extraccion de {url}")
        return 0

    save_entry(conn, "github_trending", url, "repos_trending", "Repos Trending GitHub", content)
    print(f"[SCRAPER] GitHub: guardado {len(content)} chars")
    return 1


def extract_arxiv_papers(conn: sqlite3.Connection) -> int:
    """Extrae papers recientes de arXiv cs.AI."""
    print("[SCRAPER] Escaneando arXiv cs.AI...")
    url = "https://arxiv.org/list/cs.AI/recent"
    content = extract_content(url)

    if not content:
        print(f"[SCRAPER] Fallo extraccion de {url}")
        return 0

    save_entry(conn, "arxiv_ai", url, "papers_ia", "Papers arXiv cs.AI", content)
    print(f"[SCRAPER] arXiv: guardado {len(content)} chars")
    return 1


def scan_all_sources(conn: sqlite3.Connection) -> dict:
    """Escanea todas las fuentes configuradas."""
    results = {
        "hacker_news": 0,
        "github_trending": 0,
        "arxiv_ai": 0,
        "total": 0
    }

    results["hacker_news"] = extract_hacker_news(conn)
    time.sleep(2)

    results["github_trending"] = extract_github_trending(conn)
    time.sleep(2)

    results["arxiv_ai"] = extract_arxiv_papers(conn)

    results["total"] = sum(results.values())
    return results


def query_web_knowledge(conn: sqlite3.Connection, query: str, limit: int = 5) -> List[Tuple]:
    """Consulta web_knowledge por contenido relevante."""
    cursor = conn.execute("""
        SELECT timestamp, fuente, tipo, titulo, contenido
        FROM web_knowledge
        WHERE contenido LIKE ? OR titulo LIKE ?
        ORDER BY timestamp DESC
        LIMIT ?
    """, (f"%{query}%", f"%{query}%", limit))
    return cursor.fetchall()


def get_all_entries(conn: sqlite3.Connection, limit: int = 10) -> List[Tuple]:
    """Obtiene todas las entradas de web_knowledge."""
    cursor = conn.execute("""
        SELECT id, timestamp, fuente, tipo, titulo, contenido
        FROM web_knowledge
        ORDER BY timestamp DESC
        LIMIT ?
    """, (limit,))
    return cursor.fetchall()


def main():
    """Entry point principal."""
    conn = sqlite3.connect(DB_PATH)
    init_web_knowledge_table(conn)

    if len(sys.argv) < 2:
        print("Usage: scraper.py <command> [args]")
        print("  Commands:")
        print("    scan              - Escanea todas las fuentes")
        print("    query <text>      - Busca en web_knowledge")
        print("    list [limit]      - Lista entradas recientes")
        print("    url <url>         - Extrae contenido de URL especifica")
        sys.exit(1)

    command = sys.argv[1]

    if command == "scan":
        print("[SCRAPER] Iniciando scan completo...")
        results = scan_all_sources(conn)
        print(f"[SCRAPER] Scan completado: {results['total']} entradas guardadas")
        for fuente, count in results.items():
            if fuente != "total":
                print(f"  - {fuente}: {'OK' if count else 'FALLO'}")

    elif command == "query":
        if len(sys.argv) < 3:
            print("Error: query requiere texto de busqueda")
            sys.exit(1)
        query_text = sys.argv[2]
        results = query_web_knowledge(conn, query_text)
        if results:
            print(f"Encontrados {len(results)} resultados:")
            for r in results:
                print(f"  [{r[0]}] {r[1]} - {r[2]}")
                print(f"    {r[4][:100]}...")
        else:
            print("Sin resultados")

    elif command == "list":
        limit = int(sys.argv[2]) if len(sys.argv) > 2 else 10
        entries = get_all_entries(conn, limit)
        print(f"Ultimas {len(entries)} entradas en web_knowledge:")
        for e in entries:
            print(f"  {e[0]}: {e[2]} | {e[3]} | {e[1]}")

    elif command == "url":
        if len(sys.argv) < 3:
            print("Error: url requiere direccion")
            sys.exit(1)
        target_url = sys.argv[2]
        print(f"[SCRAPER] Extrayendo: {target_url}")
        content = extract_content(target_url)
        if content:
            save_entry(conn, "custom", target_url, "custom", target_url, content)
            print(f"[SCRAPER] Guardado: {len(content)} chars")
            print(f"---Contenido---\n{content[:500]}...")
        else:
            print("[SCRAPER] Fallo extraccion")

    else:
        print(f"Comando desconocido: {command}")
        sys.exit(1)

    conn.close()


if __name__ == "__main__":
    main()