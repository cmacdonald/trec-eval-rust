"""
`trec_eval` Python interface powered by `te-rust`.
"""

from typing import Any, Iterable, List, NamedTuple, Optional, Set, Union

from trec_eval._trec_eval import EvalResult, Evaluator, __version__

ScoredDoc = NamedTuple('ScoredDoc', [('query_id', str), ('doc_id', str), ('score', float)])
Qrel = NamedTuple('Qrel', [('query_id', str), ('doc_id', str), ('relevance', int)])


def evaluate(
    qrels: Any,
    run: Any,
    measures: Optional[Union[str, Iterable[str], Set[str], List[str]]] = None,
    relevance_level: int = 1,
    complete_set_average: bool = False,
    judged_docs_only: bool = False,
    max_docs_per_topic: Optional[int] = None,
    num_docs_in_coll: int = 0,
    global_gains: Optional[str] = None,
    qid: Optional[str] = None,
    docno: Optional[str] = None,
    rel: Optional[str] = None,
    score: Optional[str] = None,
) -> EvalResult:
    """Evaluate a run against relevance judgments using trec_eval.

    Parameters
    ----------
    qrels : str, os.PathLike, Dict[str, Dict[str, int]], DataFrame, or Iterable
        Relevance judgments (file path, nested dict, DataFrame, or row iterable).
    run : str, os.PathLike, Dict[str, Dict[str, float]], DataFrame, or Iterable
        System run results (file path, nested dict, DataFrame, or row iterable).
    measures : str or list of str, optional
        Measure names or groups (e.g. ['map', 'ndcg@10', 'P.5,10']). Defaults to 'official'.
    relevance_level : int, default 1
        Minimum relevance level at which a document is considered relevant (-l).
    complete_set_average : bool, default False
        Average over all queries in qrels instead of only retrieved ones (-c).
    judged_docs_only : bool, default False
        Remove all unjudged documents before evaluating (-J).
    max_docs_per_topic : int, optional
        Maximum number of documents per topic to evaluate (-M).
    num_docs_in_coll : int, default 0
        Number of documents in the collection (-N).
    global_gains : str, optional
        Global relevance-to-gain mapping (e.g. '1=3.5,2=9.0').
    qid : str, optional
        Column name for query IDs when evaluating DataFrames.
    docno : str, optional
        Column name for document IDs when evaluating DataFrames.
    rel : str, optional
        Column name for relevance grades when qrels is a DataFrame.
    score : str, optional
        Column name for retrieval scores when run is a DataFrame.

    Returns
    -------
    EvalResult
        Rich evaluation result with mapping protocol and .aggregate() / .per_query() methods.
    """
    evaluator = Evaluator(
        qrels=qrels,
        measures=measures,
        relevance_level=relevance_level,
        complete_set_average=complete_set_average,
        judged_docs_only=judged_docs_only,
        max_docs_per_topic=max_docs_per_topic,
        num_docs_in_coll=num_docs_in_coll,
        global_gains=global_gains,
        qid=qid,
        docno=docno,
        rel=rel,
    )
    return evaluator.evaluate(run, qid=qid, docno=docno, score=score)



__all__ = [
    "__version__",
    "Evaluator",
    "EvalResult",
    "ScoredDoc",
    "Qrel",
    "evaluate",
]

