https://opentelemetry.io/ja/docs/languages/rust/getting-started/

## MongoDB インデックス利用確認ログ

以下のコマンドで、date と signage_id の複合インデックスが有効に使われていることを確認しました。

```
docker exec local-mongo mongosh --quiet --eval 'db.getSiblingDB("testdb").testcol.find({ date: { $gte: new Date("2025-05-19T00:00:00Z"), $lte: new Date("2025-05-23T23:59:59Z") }, signage_id: { $gte: 7300, $lte: 7400 } }).explain("executionStats")'
```

実行結果（抜粋）:

```
{
  explainVersion: '1',
  queryPlanner: {
    namespace: 'testdb.testcol',
    ...
    winningPlan: {
      isCached: false,
      stage: 'FETCH',
      inputStage: {
        stage: 'IXSCAN',
        keyPattern: { date: 1, signage_id: 1 },
        indexName: 'date_1_signage_id_1',
        ...
        indexBounds: {
          date: [ '[new Date(1747612800000), new Date(1748044799000)]' ],
          signage_id: [ '[7300, 7400]' ]
        }
      }
    },
    rejectedPlans: []
  },
  executionStats: {
    executionSuccess: true,
    nReturned: 78,
    executionTimeMillis: 0,
    totalKeysExamined: 121,
    totalDocsExamined: 78,
    executionStages: {
      stage: 'FETCH',
      nReturned: 78,
      inputStage: {
        stage: 'IXSCAN',
        indexName: 'date_1_signage_id_1',
        keysExamined: 121,
        ...
      }
    }
  },
  ...
}
```

- `inputStage.stage: 'IXSCAN'` および `indexName: 'date_1_signage_id_1'` から、インデックスが利用されていることが分かります。
