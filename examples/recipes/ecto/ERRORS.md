#

```sql

SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput, coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), null FROM pg_type AS t LEFT JOIN pg_type AS d ON t.typbasetype = d.oid LEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype) WHERE (t.typrelid = 0) AND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))
```

```logger
2025-11-30T03:01:52.920Z TRACE [cubesql::sql::postgres::service] [pg] New connection 10
2025-11-30T03:01:52.921Z TRACE [cubejs_native::auth] [sql auth] Request ->
2025-11-30T03:01:52.921Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 73
2025-11-30T03:01:52.921Z TRACE [cubejs_native::auth] [sql auth] Request <- CheckSQLAuthTransportResponse { password: None, superuser: false, security_context: Some(Object {}), skip_password_check: Some(true) }
2025-11-30T03:01:52.921Z TRACE [cubesql::sql::postgres::shim] RUN_ON! ....
Load Request: 48c61508-730d-41c9-9597-0a909dcd319b-span-1
--
{
  "sql": "SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,\n       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (\n  SELECT a.atttypid\n  FROM pg_attribute AS a\n  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped\n  ORDER BY a.attnum\n)\n\nFROM pg_type AS t\nLEFT JOIN pg_type AS d ON t.typbasetype = d.oid\nLEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)\nWHERE (t.typrelid = 0)\nAND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))\n\n"
}
--
{
  "securityContext": {},
  "appName": "NULL",
  "protocol": "postgres",
  "apiType": "sql"
}
2025-11-30T03:01:52.966Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 74
2025-11-30T03:01:52.966Z DEBUG [cubesql::sql::postgres::shim] Query: SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,
       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (
  SELECT a.atttypid
  FROM pg_attribute AS a
  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped
  ORDER BY a.attnum
)

FROM pg_type AS t
LEFT JOIN pg_type AS d ON t.typbasetype = d.oid
LEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)
WHERE (t.typrelid = 0)
AND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))


2025-11-30T03:01:52.969Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 75
2025-11-30T03:01:52.969Z DEBUG [cubesql::compile::parser] Parsing SQL: SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,
       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (
  SELECT a.atttypid
  FROM pg_attribute AS a
  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped
  ORDER BY a.attnum
)

FROM pg_type AS t
LEFT JOIN pg_type AS d ON t.typbasetype = d.oid
LEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)
WHERE (t.typrelid = 0)
AND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))


2025-11-30T03:01:52.973Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 76
SQL API Query Planning: 48c61508-730d-41c9-9597-0a909dcd319b-span-1
--
{
  "sql": "SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,\n       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (\n  SELECT a.atttypid\n  FROM pg_attribute AS a\n  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped\n  ORDER BY a.attnum\n)\n\nFROM pg_type AS t\nLEFT JOIN pg_type AS d ON t.typbasetype = d.oid\nLEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)\nWHERE (t.typrelid = 0)\nAND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))\n\n"
}
--
{
  "planningId": "7afae26c-d804-489f-9428-7fd94c843c56",
  "securityContext": {},
  "appName": "NULL",
  "protocol": "postgres",
  "apiType": "sql"
}
2025-11-30T03:01:52.973Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 77
2025-11-30T03:01:52.979Z WARN  [datafusion::sql::planner] ARRAY(...) is not supported yet. Replacing with scalar empty array.
2025-11-30T03:01:52.984Z DEBUG [cubesql::compile::query_engine] Initial Plan: Projection: #t.oid, #t.typname, #t.typsend, #t.typreceive, #t.typoutput, #t.typinput, coalesce(#d.typelem, #t.typelem), coalesce(#r.rngsubtype, Int64(0)), List([])
  Filter: #t.typrelid = Int64(0) AND #t.typelem = Int64(0) OR NOT #__subquery-0.Int64(1)
    Subquery: types=[Exists]
      Filter: #r.rngtypid = #t.oid OR #r.rngmultitypid = #t.oid OR #t.typbasetype != Int64(0) AND #r.rngtypid = #t.typbasetype
        CrossJoin:
          Left Join: #t.typbasetype = #d.oid
            TableScan: t projection=None
            TableScan: d projection=None
          TableScan: r projection=None
      Projection: Int64(1), alias=__subquery-0
        Filter: #s.typrelid != Int64(0) AND #s.oid = ^#t.typelem
          TableScan: s projection=None
SQL API Plan Rewrite: 48c61508-730d-41c9-9597-0a909dcd319b-span-1
{
  "planningId": "7afae26c-d804-489f-9428-7fd94c843c56",
  "securityContext": {},
  "appName": "NULL",
  "protocol": "postgres",
  "apiType": "sql"
}
2025-11-30T03:01:52.985Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 78
2025-11-30T03:01:52.990Z DEBUG [cubesql::compile::rewrite::rewriter] Iterations: [Iteration { egraph_nodes: 101, egraph_classes: 100, applied: {}, hook_time: 8.92e-7, search_time: 0.001021256, apply_time: 0.000211027, rebuild_time: 5.7987e-5, total_time: 0.001294316, data: IterInfo { debug_qtrace_eclasses: None }, n_rebuilds: 0, stop_reason: Some(Saturated) }]
2025-11-30T03:01:52.990Z DEBUG [cubesql::compile::rewrite::rewriter] Best cost: CubePlanCost {
    replacers: 0,
    penalized_ast_size_outside_wrapper: 0,
    table_scans: 4,
    empty_wrappers: 0,
    non_detected_cube_scans: 0,
    unwrapped_subqueries: 1,
    member_errors: 0,
    ungrouped_aggregates: 0,
    non_pushed_down_window: 0,
    non_pushed_down_grouping_sets: 0,
    non_pushed_down_limit_sort: 0,
    joins: 2,
    wrapper_nodes: 0,
    ast_size_outside_wrapper: 8,
    wrapped_select_non_push_to_cube: 0,
    wrapped_select_ungrouped_scan: 0,
    filters: 3,
    structure_points: 2,
    zero_members_wrapper: 0,
    filter_members: 0,
    cube_members: 0,
    errors: 0,
    time_dimensions_used_as_dimensions: 0,
    max_time_dimensions_granularity: 0,
    cube_scan_nodes: 0,
    ast_size_without_alias: 140,
    ast_size: 140,
    ast_size_inside_wrapper: 0,
    ungrouped_nodes: 0,
}
2025-11-30T03:01:52.991Z DEBUG [cubesql::compile::rewrite::rewriter] Best: (Projection
  (ProjectionExpr
    (ColumnExpr #t.oid)
    (ColumnExpr #t.typname)
    (ColumnExpr #t.typsend)
    (ColumnExpr #t.typreceive)
    (ColumnExpr #t.typoutput)
    (ColumnExpr #t.typinput)
    (ScalarFunctionExpr Coalesce (ScalarFunctionExprArgs (ColumnExpr #d.typelem) (ColumnExpr #t.typelem)))
    (ScalarFunctionExpr
      Coalesce
      (ScalarFunctionExprArgs (ColumnExpr #r.rngsubtype) (LiteralExpr LiteralExprValue(Int64(0)))))
    (LiteralExpr LiteralExprValue(List([]))))
  (Filter
    (BinaryExpr
      (BinaryExpr (ColumnExpr #t.typrelid) = (LiteralExpr LiteralExprValue(Int64(0))))
      AND
      (BinaryExpr
        (BinaryExpr (ColumnExpr #t.typelem) = (LiteralExpr LiteralExprValue(Int64(0))))
        OR
        (NotExpr (ColumnExpr #__subquery-0.Int64(1)))))
    (Subquery
      (Filter
        (BinaryExpr
          (BinaryExpr
            (BinaryExpr (ColumnExpr #r.rngtypid) = (ColumnExpr #t.oid))
            OR
            (BinaryExpr (ColumnExpr #r.rngmultitypid) = (ColumnExpr #t.oid)))
          OR
          (BinaryExpr
            (BinaryExpr (ColumnExpr #t.typbasetype) != (LiteralExpr LiteralExprValue(Int64(0))))
            AND
            (BinaryExpr (ColumnExpr #r.rngtypid) = (ColumnExpr #t.typbasetype))))
        (CrossJoin
          (Join
            (TableScan pg_catalog.pg_type t TableScanProjection(None) TableScanFilters None)
            (TableScan pg_catalog.pg_type d TableScanProjection(None) TableScanFilters None)
            [Column { relation: Some("t"), name: "typbasetype" }]
            [Column { relation: Some("d"), name: "oid" }]
            Left
            On
            false)
          (TableScan pg_catalog.pg_range r TableScanProjection(None) TableScanFilters None)))
      (SubquerySubqueries
        (Projection
          (ProjectionExpr (LiteralExpr LiteralExprValue(Int64(1))))
          (Filter
            (BinaryExpr
              (BinaryExpr (ColumnExpr #s.typrelid) != (LiteralExpr LiteralExprValue(Int64(0))))
              AND
              (BinaryExpr (ColumnExpr #s.oid) = (OuterColumnExpr UInt32 #t.typelem)))
            (TableScan pg_catalog.pg_type s TableScanProjection(None) TableScanFilters None))
          Some("__subquery-0")
          false)
        SubquerySubqueries)
      SubqueryTypes([Exists])))
  None
  false)
SQL API Plan Rewrite Success: 48c61508-730d-41c9-9597-0a909dcd319b-span-1 (7ms)
{
  "planningId": "7afae26c-d804-489f-9428-7fd94c843c56",
  "securityContext": {},
  "appName": "NULL",
  "protocol": "postgres",
  "apiType": "sql"
}
2025-11-30T03:01:52.993Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 79
2025-11-30T03:01:52.993Z DEBUG [cubesql::compile::query_engine] Rewrite: Projection: #t.oid, #t.typname, #t.typsend, #t.typreceive, #t.typoutput, #t.typinput, coalesce(#d.typelem, #t.typelem), coalesce(#r.rngsubtype, Int64(0)), List([])
  Filter: #t.typrelid = Int64(0) AND #t.typelem = Int64(0) OR NOT #__subquery-0.Int64(1)
    Subquery: types=[Exists]
      Filter: #r.rngtypid = #t.oid OR #r.rngmultitypid = #t.oid OR #t.typbasetype != Int64(0) AND #r.rngtypid = #t.typbasetype
        CrossJoin:
          Left Join: #t.typbasetype = #d.oid
            TableScan: t projection=None
            TableScan: d projection=None
          TableScan: r projection=None
      Projection: Int64(1), alias=__subquery-0
        Filter: #s.typrelid != Int64(0) AND #s.oid = ^#t.typelem
          TableScan: s projection=None
SQL API Query Planning Success: 48c61508-730d-41c9-9597-0a909dcd319b-span-1 (20ms)
--
{
  "sql": "SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,\n       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (\n  SELECT a.atttypid\n  FROM pg_attribute AS a\n  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped\n  ORDER BY a.attnum\n)\n\nFROM pg_type AS t\nLEFT JOIN pg_type AS d ON t.typbasetype = d.oid\nLEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)\nWHERE (t.typrelid = 0)\nAND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))\n\n"
}
--
{
  "planningId": "7afae26c-d804-489f-9428-7fd94c843c56",
  "securityContext": {},
  "appName": "NULL",
  "protocol": "postgres",
  "apiType": "sql"
}
2025-11-30T03:01:52.993Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 80
2025-11-30T03:01:52.995Z TRACE [cubesql::sql::postgres::extended] execute; self=Portal { format: Text, from: Simple, state: Some(Prepared(PreparedState { plan: DataFusionSelect(LogicalPlan: hidden, DFSessionContext: hidden) })), span_id: Some(SpanId { span_id: "48c61508-730d-41c9-9597-0a909dcd319b", query_key: Object {"sql": String("SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,\n       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (\n  SELECT a.atttypid\n  FROM pg_attribute AS a\n  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped\n  ORDER BY a.attnum\n)\n\nFROM pg_type AS t\nLEFT JOIN pg_type AS d ON t.typbasetype = d.oid\nLEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)\nWHERE (t.typrelid = 0)\nAND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))\n\n")}, span_start: SystemTime { tv_sec: 1764471712, tv_nsec: 962908419 }, is_data_query: RwLock { data: false } }) } max_rows=0
Load Request Success: 48c61508-730d-41c9-9597-0a909dcd319b-span-1 (74ms)
--
{
  "sql": "SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,\n       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (\n  SELECT a.atttypid\n  FROM pg_attribute AS a\n  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped\n  ORDER BY a.attnum\n)\n\nFROM pg_type AS t\nLEFT JOIN pg_type AS d ON t.typbasetype = d.oid\nLEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)\nWHERE (t.typrelid = 0)\nAND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))\n\n"
}
--
{
  "apiType": "sql",
  "isDataQuery": false,
  "securityContext": {},
  "appName": "NULL",
  "protocol": "postgres"
}
2025-11-30T03:01:53.038Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 81
2025-11-30T03:01:53.069Z TRACE [cubesql::sql::postgres::shim] MIne GOD: Error during processing PostgreSQL connection: ProtocolError: IO error: unexpected end of file
2025-11-30T03:01:53.069Z TRACE [cubesql::sql::postgres::service] [pg] Removing connection 10
2025-11-30T03:01:53.073Z TRACE [cubesql::sql::postgres::service] [pg] New connection 11
2025-11-30T03:01:53.074Z TRACE [cubejs_native::auth] [sql auth] Request ->
2025-11-30T03:01:53.075Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 82
2025-11-30T03:01:53.075Z TRACE [cubejs_native::auth] [sql auth] Request <- CheckSQLAuthTransportResponse { password: None, superuser: false, security_context: Some(Object {}), skip_password_check: Some(true) }
2025-11-30T03:01:53.075Z TRACE [cubesql::sql::postgres::shim] RUN_ON! ....
Load Request: e0117437-0858-4bee-8a8c-922a62594b95-span-1
--
{
  "sql": "SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,\n       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (\n  SELECT a.atttypid\n  FROM pg_attribute AS a\n  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped\n  ORDER BY a.attnum\n)\n\nFROM pg_type AS t\nLEFT JOIN pg_type AS d ON t.typbasetype = d.oid\nLEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)\nWHERE (t.typrelid = 0)\nAND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))\n\n"
}
--
{
  "securityContext": {},
  "appName": "NULL",
  "protocol": "postgres",
  "apiType": "sql"
}
2025-11-30T03:01:53.117Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 83
2025-11-30T03:01:53.117Z DEBUG [cubesql::sql::postgres::shim] Query: SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,
       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (
  SELECT a.atttypid
  FROM pg_attribute AS a
  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped
  ORDER BY a.attnum
)

FROM pg_type AS t
LEFT JOIN pg_type AS d ON t.typbasetype = d.oid
LEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)
WHERE (t.typrelid = 0)
AND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))


2025-11-30T03:01:53.122Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 84
2025-11-30T03:01:53.122Z DEBUG [cubesql::compile::parser] Parsing SQL: SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,
       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (
  SELECT a.atttypid
  FROM pg_attribute AS a
  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped
  ORDER BY a.attnum
)

FROM pg_type AS t
LEFT JOIN pg_type AS d ON t.typbasetype = d.oid
LEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)
WHERE (t.typrelid = 0)
AND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))


2025-11-30T03:01:53.125Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 85
SQL API Query Planning: e0117437-0858-4bee-8a8c-922a62594b95-span-1
--
{
  "sql": "SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,\n       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (\n  SELECT a.atttypid\n  FROM pg_attribute AS a\n  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped\n  ORDER BY a.attnum\n)\n\nFROM pg_type AS t\nLEFT JOIN pg_type AS d ON t.typbasetype = d.oid\nLEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)\nWHERE (t.typrelid = 0)\nAND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))\n\n"
}
--
{
  "planningId": "05ff7c44-4f18-4d87-b600-d8b402da6d2e",
  "securityContext": {},
  "appName": "NULL",
  "protocol": "postgres",
  "apiType": "sql"
}
2025-11-30T03:01:53.126Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 86
2025-11-30T03:01:53.131Z WARN  [datafusion::sql::planner] ARRAY(...) is not supported yet. Replacing with scalar empty array.
2025-11-30T03:01:53.136Z DEBUG [cubesql::compile::query_engine] Initial Plan: Projection: #t.oid, #t.typname, #t.typsend, #t.typreceive, #t.typoutput, #t.typinput, coalesce(#d.typelem, #t.typelem), coalesce(#r.rngsubtype, Int64(0)), List([])
  Filter: #t.typrelid = Int64(0) AND #t.typelem = Int64(0) OR NOT #__subquery-0.Int64(1)
    Subquery: types=[Exists]
      Filter: #r.rngtypid = #t.oid OR #r.rngmultitypid = #t.oid OR #t.typbasetype != Int64(0) AND #r.rngtypid = #t.typbasetype
        CrossJoin:
          Left Join: #t.typbasetype = #d.oid
            TableScan: t projection=None
            TableScan: d projection=None
          TableScan: r projection=None
      Projection: Int64(1), alias=__subquery-0
        Filter: #s.typrelid != Int64(0) AND #s.oid = ^#t.typelem
          TableScan: s projection=None
SQL API Plan Rewrite: e0117437-0858-4bee-8a8c-922a62594b95-span-1
{
  "planningId": "05ff7c44-4f18-4d87-b600-d8b402da6d2e",
  "securityContext": {},
  "appName": "NULL",
  "protocol": "postgres",
  "apiType": "sql"
}
2025-11-30T03:01:53.137Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 87
2025-11-30T03:01:53.139Z DEBUG [cubesql::compile::rewrite::rewriter] Iterations: [Iteration { egraph_nodes: 101, egraph_classes: 100, applied: {}, hook_time: 1.985e-6, search_time: 0.000775345, apply_time: 0.000149847, rebuild_time: 5.7736e-5, total_time: 0.000987016, data: IterInfo { debug_qtrace_eclasses: None }, n_rebuilds: 0, stop_reason: Some(Saturated) }]
2025-11-30T03:01:53.140Z DEBUG [cubesql::compile::rewrite::rewriter] Best cost: CubePlanCost {
    replacers: 0,
    penalized_ast_size_outside_wrapper: 0,
    table_scans: 4,
    empty_wrappers: 0,
    non_detected_cube_scans: 0,
    unwrapped_subqueries: 1,
    member_errors: 0,
    ungrouped_aggregates: 0,
    non_pushed_down_window: 0,
    non_pushed_down_grouping_sets: 0,
    non_pushed_down_limit_sort: 0,
    joins: 2,
    wrapper_nodes: 0,
    ast_size_outside_wrapper: 8,
    wrapped_select_non_push_to_cube: 0,
    wrapped_select_ungrouped_scan: 0,
    filters: 3,
    structure_points: 2,
    zero_members_wrapper: 0,
    filter_members: 0,
    cube_members: 0,
    errors: 0,
    time_dimensions_used_as_dimensions: 0,
    max_time_dimensions_granularity: 0,
    cube_scan_nodes: 0,
    ast_size_without_alias: 140,
    ast_size: 140,
    ast_size_inside_wrapper: 0,
    ungrouped_nodes: 0,
}
2025-11-30T03:01:53.140Z DEBUG [cubesql::compile::rewrite::rewriter] Best: (Projection
  (ProjectionExpr
    (ColumnExpr #t.oid)
    (ColumnExpr #t.typname)
    (ColumnExpr #t.typsend)
    (ColumnExpr #t.typreceive)
    (ColumnExpr #t.typoutput)
    (ColumnExpr #t.typinput)
    (ScalarFunctionExpr Coalesce (ScalarFunctionExprArgs (ColumnExpr #d.typelem) (ColumnExpr #t.typelem)))
    (ScalarFunctionExpr
      Coalesce
      (ScalarFunctionExprArgs (ColumnExpr #r.rngsubtype) (LiteralExpr LiteralExprValue(Int64(0)))))
    (LiteralExpr LiteralExprValue(List([]))))
  (Filter
    (BinaryExpr
      (BinaryExpr (ColumnExpr #t.typrelid) = (LiteralExpr LiteralExprValue(Int64(0))))
      AND
      (BinaryExpr
        (BinaryExpr (ColumnExpr #t.typelem) = (LiteralExpr LiteralExprValue(Int64(0))))
        OR
        (NotExpr (ColumnExpr #__subquery-0.Int64(1)))))
    (Subquery
      (Filter
        (BinaryExpr
          (BinaryExpr
            (BinaryExpr (ColumnExpr #r.rngtypid) = (ColumnExpr #t.oid))
            OR
            (BinaryExpr (ColumnExpr #r.rngmultitypid) = (ColumnExpr #t.oid)))
          OR
          (BinaryExpr
            (BinaryExpr (ColumnExpr #t.typbasetype) != (LiteralExpr LiteralExprValue(Int64(0))))
            AND
            (BinaryExpr (ColumnExpr #r.rngtypid) = (ColumnExpr #t.typbasetype))))
        (CrossJoin
          (Join
            (TableScan pg_catalog.pg_type t TableScanProjection(None) TableScanFilters None)
            (TableScan pg_catalog.pg_type d TableScanProjection(None) TableScanFilters None)
            [Column { relation: Some("t"), name: "typbasetype" }]
            [Column { relation: Some("d"), name: "oid" }]
            Left
            On
            false)
          (TableScan pg_catalog.pg_range r TableScanProjection(None) TableScanFilters None)))
      (SubquerySubqueries
        (Projection
          (ProjectionExpr (LiteralExpr LiteralExprValue(Int64(1))))
          (Filter
            (BinaryExpr
              (BinaryExpr (ColumnExpr #s.typrelid) != (LiteralExpr LiteralExprValue(Int64(0))))
              AND
              (BinaryExpr (ColumnExpr #s.oid) = (OuterColumnExpr UInt32 #t.typelem)))
            (TableScan pg_catalog.pg_type s TableScanProjection(None) TableScanFilters None))
          Some("__subquery-0")
          false)
        SubquerySubqueries)
      SubqueryTypes([Exists])))
  None
  false)
SQL API Plan Rewrite Success: e0117437-0858-4bee-8a8c-922a62594b95-span-1 (4ms)
{
  "planningId": "05ff7c44-4f18-4d87-b600-d8b402da6d2e",
  "securityContext": {},
  "appName": "NULL",
  "protocol": "postgres",
  "apiType": "sql"
}
2025-11-30T03:01:53.142Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 88
2025-11-30T03:01:53.142Z DEBUG [cubesql::compile::query_engine] Rewrite: Projection: #t.oid, #t.typname, #t.typsend, #t.typreceive, #t.typoutput, #t.typinput, coalesce(#d.typelem, #t.typelem), coalesce(#r.rngsubtype, Int64(0)), List([])
  Filter: #t.typrelid = Int64(0) AND #t.typelem = Int64(0) OR NOT #__subquery-0.Int64(1)
    Subquery: types=[Exists]
      Filter: #r.rngtypid = #t.oid OR #r.rngmultitypid = #t.oid OR #t.typbasetype != Int64(0) AND #r.rngtypid = #t.typbasetype
        CrossJoin:
          Left Join: #t.typbasetype = #d.oid
            TableScan: t projection=None
            TableScan: d projection=None
          TableScan: r projection=None
      Projection: Int64(1), alias=__subquery-0
        Filter: #s.typrelid != Int64(0) AND #s.oid = ^#t.typelem
          TableScan: s projection=None
SQL API Query Planning Success: e0117437-0858-4bee-8a8c-922a62594b95-span-1 (16ms)
--
{
  "sql": "SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,\n       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (\n  SELECT a.atttypid\n  FROM pg_attribute AS a\n  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped\n  ORDER BY a.attnum\n)\n\nFROM pg_type AS t\nLEFT JOIN pg_type AS d ON t.typbasetype = d.oid\nLEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)\nWHERE (t.typrelid = 0)\nAND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))\n\n"
}
--
{
  "planningId": "05ff7c44-4f18-4d87-b600-d8b402da6d2e",
  "securityContext": {},
  "appName": "NULL",
  "protocol": "postgres",
  "apiType": "sql"
}
2025-11-30T03:01:53.143Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 89
2025-11-30T03:01:53.144Z TRACE [cubesql::sql::postgres::extended] execute; self=Portal { format: Text, from: Simple, state: Some(Prepared(PreparedState { plan: DataFusionSelect(LogicalPlan: hidden, DFSessionContext: hidden) })), span_id: Some(SpanId { span_id: "e0117437-0858-4bee-8a8c-922a62594b95", query_key: Object {"sql": String("SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,\n       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (\n  SELECT a.atttypid\n  FROM pg_attribute AS a\n  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped\n  ORDER BY a.attnum\n)\n\nFROM pg_type AS t\nLEFT JOIN pg_type AS d ON t.typbasetype = d.oid\nLEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)\nWHERE (t.typrelid = 0)\nAND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))\n\n")}, span_start: SystemTime { tv_sec: 1764471713, tv_nsec: 117115691 }, is_data_query: RwLock { data: false } }) } max_rows=0
Load Request Success: e0117437-0858-4bee-8a8c-922a62594b95-span-1 (71ms)
--
{
  "sql": "SELECT t.oid, t.typname, t.typsend, t.typreceive, t.typoutput, t.typinput,\n       coalesce(d.typelem, t.typelem), coalesce(r.rngsubtype, 0), ARRAY (\n  SELECT a.atttypid\n  FROM pg_attribute AS a\n  WHERE a.attrelid = t.typrelid AND a.attnum > 0 AND NOT a.attisdropped\n  ORDER BY a.attnum\n)\n\nFROM pg_type AS t\nLEFT JOIN pg_type AS d ON t.typbasetype = d.oid\nLEFT JOIN pg_range AS r ON r.rngtypid = t.oid OR r.rngmultitypid = t.oid OR (t.typbasetype <> 0 AND r.rngtypid = t.typbasetype)\nWHERE (t.typrelid = 0)\nAND (t.typelem = 0 OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_type s WHERE s.typrelid != 0 AND s.oid = t.typelem))\n\n"
}
--
{
  "apiType": "sql",
  "isDataQuery": false,
  "securityContext": {},
  "appName": "NULL",
  "protocol": "postgres"
}
2025-11-30T03:01:53.189Z TRACE [cubejs_native::channel] JsAsyncChannel.resolved 90
2025-11-30T03:01:53.196Z TRACE [cubesql::sql::postgres::shim] MIne GOD: Error during processing PostgreSQL connection: ProtocolError: IO error: unexpected end of file
2025-11-30T03:01:53.196Z TRACE [cubesql::sql::postgres::service] [pg] Removing connection 11
Refresh Scheduler
```
