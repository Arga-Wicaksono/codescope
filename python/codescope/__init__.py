"""
CodeScope Python SDK

Programmatic Python interface for CodeScope repository intelligence.

Usage:
    from codescope import CodeScope

    cs = CodeScope("/path/to/repo")

    # Search files
    files = cs.search_files("config")

    # Search content
    results = cs.search_content("fn main", extensions=["rs"])

    # Find symbols
    symbols = cs.find_symbols("authenticate")

    # Get stats
    stats = cs.stats()

    # Dependency graph
    graph = cs.dependency_graph()

    # Impact analysis
    impact = cs.impact("auth.rs")

    # Context extraction
    context = cs.get_context("authentication")
"""

import subprocess
import json
import os
from typing import Optional, List, Dict, Any
from dataclasses import dataclass, field, asdict
from pathlib import Path


class CodeScopeError(Exception):
    """Base exception for CodeScope SDK errors."""
    pass


class BinaryNotFoundError(CodeScopeError):
    """Raised when the cs binary is not found."""
    pass


class SearchError(CodeScopeError):
    """Raised when a search operation fails."""
    pass


@dataclass
class FileResult:
    """A file search result."""
    path: str
    extension: str
    size: int
    score: int = 0

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class ContentResult:
    """A content search result."""
    file: str
    line: int
    content: str

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class SymbolResult:
    """A symbol search result."""
    name: str
    kind: str
    file: str
    line: int
    snippet: str = ""
    language: str = ""

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class LanguageStats:
    """Statistics for a single language."""
    language: str
    files: int
    lines: int
    bytes: int

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class RepoStats:
    """Repository statistics."""
    total_files: int
    total_lines: int
    by_language: List[LanguageStats] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "total_files": self.total_files,
            "total_lines": self.total_lines,
            "by_language": [lang.to_dict() for lang in self.by_language],
        }


@dataclass
class GraphNode:
    """A dependency graph node."""
    name: str
    kind: str
    language: str
    path: str
    loc: int

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class GraphEdge:
    """A dependency graph edge."""
    from_node: str
    to_node: str
    kind: str
    weight: float

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class DependencyGraph:
    """A dependency graph."""
    nodes: List[GraphNode] = field(default_factory=list)
    edges: List[GraphEdge] = field(default_factory=list)
    total_nodes: int = 0
    total_edges: int = 0

    def to_dict(self) -> Dict[str, Any]:
        return {
            "nodes": [n.to_dict() for n in self.nodes],
            "edges": [e.to_dict() for e in self.edges],
            "total_nodes": self.total_nodes,
            "total_edges": self.total_edges,
        }

    def to_dot(self) -> str:
        """Export the graph as Graphviz DOT format."""
        lines = ["digraph dependencies {", "  rankdir=LR;", "  node [shape=box];", ""]
        for node in self.nodes:
            lines.append(f'  "{node.path}" [label="{node.name}"];')
        lines.append("")
        for edge in self.edges:
            color = "#4CAF50" if edge.kind == "imports" else "#2196F3"
            lines.append(f'  "{edge.from_node}" -> "{edge.to_node}" [color="{color}"];')
        lines.append("}")
        return "\n".join(lines)


@dataclass
class ImpactResult:
    """Impact analysis result."""
    target: str
    direct_dependents: List[str] = field(default_factory=list)
    transitive_dependents: List[str] = field(default_factory=list)
    total_affected: int = 0

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class ContextResult:
    """Context extraction result."""
    topic: str
    files: List[str] = field(default_factory=list)
    symbols: List[SymbolResult] = field(default_factory=list)
    total_tokens: int = 0

    def to_dict(self) -> Dict[str, Any]:
        return {
            "topic": self.topic,
            "files": self.files,
            "symbols": [s.to_dict() for s in self.symbols],
            "total_tokens": self.total_tokens,
        }


class CodeScope:
    """
    CodeScope Python SDK.

    Provides programmatic access to CodeScope repository intelligence
    capabilities: file search, content search, symbol intelligence,
    context extraction, dependency graphs, and impact analysis.

    Args:
        repo_path: Path to the repository root directory.
        cs_binary: Optional path to the cs binary. If not provided,
                   looks for 'cs' in PATH.

    Raises:
        BinaryNotFoundError: If the cs binary cannot be found.
        CodeScopeError: If the repo path is invalid.

    Example:
        >>> cs = CodeScope("/path/to/repo")
        >>> files = cs.search_files("config")
        >>> for f in files:
        ...     print(f.path, f.score)
    """

    def __init__(self, repo_path: str, cs_binary: Optional[str] = None):
        self.repo_path = os.path.abspath(repo_path)

        if not os.path.isdir(self.repo_path):
            raise CodeScopeError(f"Not a directory: {self.repo_path}")

        self.cs_binary = cs_binary or self._find_binary()

    def _find_binary(self) -> str:
        """Find the cs binary in PATH."""
        try:
            result = subprocess.run(
                ["which", "cs"],
                capture_output=True,
                text=True,
                timeout=5,
            )
            if result.returncode == 0:
                return result.stdout.strip()
        except (FileNotFoundError, subprocess.TimeoutExpired):
            pass

        # Try common locations
        common_paths = [
            "/usr/local/bin/cs",
            os.path.expanduser("~/.cargo/bin/cs"),
            os.path.expanduser("~/.local/bin/cs"),
        ]
        for path in common_paths:
            if os.path.isfile(path) and os.access(path, os.X_OK):
                return path

        raise BinaryNotFoundError(
            "cs binary not found. Install CodeScope or set cs_binary parameter. "
            "See: https://github.com/Arga-Wicaksono/codescope"
        )

    def _run(self, args: List[str], timeout: int = 30) -> str:
        """Run a cs command and return JSON output."""
        cmd = [self.cs_binary] + args + ["-j"]
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=timeout,
                cwd=self.repo_path,
            )
            if result.returncode != 0 and result.returncode != 1:
                raise SearchError(f"cs command failed: {result.stderr.strip()}")
            return result.stdout.strip()
        except subprocess.TimeoutExpired:
            raise SearchError(f"Command timed out after {timeout}s: {' '.join(cmd)}")

    def search_files(self, pattern: str, extension: Optional[str] = None,
                     limit: int = 50) -> List[FileResult]:
        """
        Search for files by name using fuzzy matching.

        Args:
            pattern: Search pattern (fuzzy matching).
            extension: Optional file extension filter (e.g., "rs", "py").
            limit: Maximum number of results.

        Returns:
            List of FileResult objects.

        Example:
            >>> files = cs.search_files("config", extension="rs")
        """
        args = ["file", pattern, "-l", str(limit)]
        if extension:
            args.extend(["-e", extension])

        output = self._run(args)
        if not output:
            return []

        data = json.loads(output)
        results = []
        for item in data.get("results", []):
            results.append(FileResult(
                path=item.get("path", ""),
                extension=item.get("extension", ""),
                size=item.get("size", 0),
                score=item.get("score", 0),
            ))
        return results

    def search_content(self, pattern: str, extensions: Optional[List[str]] = None,
                       line_numbers: bool = True, context: int = 0,
                       limit: int = 50, exact: bool = False,
                       regex: bool = False) -> List[ContentResult]:
        """
        Search for content inside files.

        Args:
            pattern: Search pattern.
            extensions: Optional list of file extensions to search.
            line_numbers: Whether to include line numbers.
            context: Number of context lines around matches.
            limit: Maximum number of results.
            exact: Use exact substring matching.
            regex: Use regex matching.

        Returns:
            List of ContentResult objects.

        Example:
            >>> results = cs.search_content("fn main", extensions=["rs"])
        """
        args = ["content", pattern, "-l", str(limit)]
        if line_numbers:
            args.append("-n")
        if context > 0:
            args.extend(["-C", str(context)])
        if exact:
            args.append("-x")
        if regex:
            args.append("--regex")

        output = self._run(args)
        if not output:
            return []

        data = json.loads(output)
        results = []
        for item in data.get("results", []):
            results.append(ContentResult(
                file=item.get("file", ""),
                line=item.get("line", 0),
                content=item.get("content", ""),
            ))
        return results

    def find_symbol(self, name: str, kind: Optional[str] = None,
                    limit: int = 50) -> List[SymbolResult]:
        """
        Find symbol definitions by name.

        Args:
            name: Symbol name to search for.
            kind: Optional symbol kind filter (function, class, struct, etc.).
            limit: Maximum number of results.

        Returns:
            List of SymbolResult objects.

        Example:
            >>> symbols = cs.find_symbol("authenticate", kind="function")
        """
        args = ["symbol", name, "-l", str(limit)]
        if kind:
            args.extend(["--symbol-type", kind])

        output = self._run(args)
        if not output:
            return []

        data = json.loads(output)
        results = []
        for item in data.get("results", []):
            results.append(SymbolResult(
                name=item.get("name", ""),
                kind=item.get("kind", ""),
                file=item.get("file", ""),
                line=item.get("line", 0),
                snippet=item.get("snippet", ""),
                language=item.get("language", ""),
            ))
        return results

    def find_references(self, name: str, limit: int = 50) -> List[ContentResult]:
        """
        Find all references to a symbol.

        Args:
            name: Symbol name.
            limit: Maximum number of results.

        Returns:
            List of ContentResult objects.
        """
        args = ["refs", name, "-l", str(limit)]
        output = self._run(args)
        if not output:
            return []

        data = json.loads(output)
        results = []
        for item in data.get("results", []):
            results.append(ContentResult(
                file=item.get("file", ""),
                line=item.get("line", 0),
                content=item.get("content", ""),
            ))
        return results

    def find_callers(self, name: str, limit: int = 50) -> List[ContentResult]:
        """
        Find all callers of a function.

        Args:
            name: Function name.
            limit: Maximum number of results.

        Returns:
            List of ContentResult objects.
        """
        args = ["callers", name, "-l", str(limit)]
        output = self._run(args)
        if not output:
            return []

        data = json.loads(output)
        results = []
        for item in data.get("results", []):
            results.append(ContentResult(
                file=item.get("file", ""),
                line=item.get("line", 0),
                content=item.get("content", ""),
            ))
        return results

    def list_symbols(self, path: Optional[str] = None, kind: Optional[str] = None,
                     limit: int = 100) -> List[SymbolResult]:
        """
        List all symbols in a file or directory.

        Args:
            path: File or directory path (default: repository root).
            kind: Optional symbol kind filter.
            limit: Maximum number of results.

        Returns:
            List of SymbolResult objects.
        """
        args = ["symbols", "-l", str(limit)]
        if path:
            args.extend(["-p", path])
        if kind:
            args.extend(["--symbol-type", kind])

        output = self._run(args)
        if not output:
            return []

        data = json.loads(output)
        results = []
        for item in data.get("results", []):
            results.append(SymbolResult(
                name=item.get("name", ""),
                kind=item.get("kind", ""),
                file=item.get("file", ""),
                line=item.get("line", 0),
                snippet=item.get("snippet", ""),
                language=item.get("language", ""),
            ))
        return results

    def stats(self, file_type: Optional[str] = None) -> RepoStats:
        """
        Get repository statistics.

        Args:
            file_type: Optional file type preset (rust, python, web, etc.).

        Returns:
            RepoStats object.

        Example:
            >>> stats = cs.stats()
            >>> print(f"Total: {stats.total_files} files, {stats.total_lines} lines")
        """
        args = ["stats"]
        if file_type:
            args.extend(["--type", file_type])

        output = self._run(args)
        if not output:
            return RepoStats(total_files=0, total_lines=0)

        data = json.loads(output)
        by_language = []
        for item in data.get("results", []):
            by_language.append(LanguageStats(
                language=item.get("language", ""),
                files=item.get("files", 0),
                lines=item.get("lines", 0),
                bytes=item.get("bytes", 0),
            ))

        return RepoStats(
            total_files=data.get("total_files", 0),
            total_lines=data.get("total_lines", 0),
            by_language=by_language,
        )

    def dependency_graph(self, graph_type: str = "modules") -> DependencyGraph:
        """
        Build the dependency graph.

        Args:
            graph_type: "modules" for import graph, "calls" for call graph.

        Returns:
            DependencyGraph object.

        Example:
            >>> graph = cs.dependency_graph()
            >>> print(graph.to_dot())  # Export to Graphviz
        """
        args = ["graph", "-t", graph_type, "-j"]
        output = self._run(args)
        if not output:
            return DependencyGraph()

        data = json.loads(output)
        nodes = []
        for item in data.get("nodes", []):
            nodes.append(GraphNode(
                name=item.get("name", ""),
                kind=item.get("kind", ""),
                language=item.get("language", ""),
                path=item.get("path", ""),
                loc=item.get("loc", 0),
            ))

        edges = []
        for item in data.get("edges", []):
            edges.append(GraphEdge(
                from_node=item.get("from", ""),
                to_node=item.get("to", ""),
                kind=item.get("kind", ""),
                weight=item.get("weight", 1.0),
            ))

        return DependencyGraph(
            nodes=nodes,
            edges=edges,
            total_nodes=data.get("total_nodes", 0),
            total_edges=data.get("total_edges", 0),
        )

    def impact(self, target: str) -> ImpactResult:
        """
        Analyze the impact of modifying a file or module.

        Args:
            target: File or module name to analyze.

        Returns:
            ImpactResult object.

        Example:
            >>> impact = cs.impact("auth.rs")
            >>> print(f"Affected: {impact.total_affected} files")
        """
        args = ["impact", target, "-j"]
        output = self._run(args)
        if not output:
            return ImpactResult(target=target)

        data = json.loads(output)
        return ImpactResult(
            target=data.get("target", target),
            direct_dependents=data.get("direct_dependents", []),
            transitive_dependents=data.get("transitive_dependents", []),
            total_affected=data.get("total_affected", 0),
        )

    def get_context(self, topic: str, max_items: int = 20) -> ContextResult:
        """
        Extract context for a topic.

        Args:
            topic: Topic to extract context for.
            max_items: Maximum number of context items.

        Returns:
            ContextResult object.

        Example:
            >>> ctx = cs.get_context("authentication")
            >>> for f in ctx.files:
            ...     print(f)
        """
        args = ["context", topic, "-l", str(max_items), "-j"]
        output = self._run(args)
        if not output:
            return ContextResult(topic=topic)

        data = json.loads(output)
        return ContextResult(
            topic=topic,
            files=data.get("files", []),
            total_tokens=data.get("total_tokens", 0),
        )

    def pack_context(self, description: str, budget: int = 8000) -> str:
        """
        Pack context into a token-efficient format for LLM prompts.

        Args:
            description: Description of what context is needed.
            budget: Token budget.

        Returns:
            Packed context string.
        """
        args = ["pack", description, "-b", str(budget), "-j"]
        output = self._run(args)
        if not output:
            return ""

        data = json.loads(output)
        return data.get("packed", "")

    def trace(self, symbol: str, max_depth: int = 5) -> str:
        """
        Trace execution flow through function calls.

        Args:
            symbol: Symbol name to start tracing from.
            max_depth: Maximum trace depth.

        Returns:
            Trace result as JSON string.
        """
        args = ["trace", symbol, "-d", str(max_depth), "-j"]
        return self._run(args)

    def __repr__(self) -> str:
        return f"CodeScope(repo_path={self.repo_path!r})"
