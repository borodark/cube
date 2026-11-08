import crypto from 'crypto';
import { Readable } from 'stream';
import type { QueryKey } from '@cubejs-backend/base-driver';

import { QueryQueue } from '../../src';
import type { QueryStream } from '../../src/orchestrator/QueryStream';

class TestQueryQueue extends QueryQueue {
  public redisHash(queryKey: QueryKey) {
    return super.redisHash(queryKey);
  }
}

// A queue that skips queueing, as the one for Cube Store does: a streamed query
// goes straight to the stream handler, which pipes the driver's rows in.
describe('QueryQueue with skipQueue, streamed queries', () => {
  const createQueue = (streamHandler?: (q: any, target: QueryStream) => Promise<unknown>) => new TestQueryQueue(
    `${crypto.randomBytes(6).toString('hex')}#skip_queue_stream`,
    {
      cacheAndQueueDriver: 'memory',
      skipQueue: true,
      queryHandlers: { query: async () => [] },
      cancelHandlers: {},
      streamHandler,
      logger: jest.fn(),
    }
  );

  /** A stream handler that pipes `rows` in, as QueryCache's does with a driver's row stream. */
  const piping = (rows: () => Readable) => jest.fn(async (_q: any, target: QueryStream) => {
    await new Promise((resolve) => {
      const source = rows();
      source.once('error', (error) => target.destroy(error));
      target.once('close', resolve);
      source.pipe(target);
    });
  });

  const key = (sql: string): QueryKey => {
    const k: QueryKey = [sql, []];
    k.persistent = true;
    return k;
  };

  const readAll = async (s: AsyncIterable<unknown>) => {
    const rows: unknown[] = [];

    // eslint-disable-next-line no-restricted-syntax
    for await (const row of s) {
      rows.push(row);
    }
    return rows;
  };

  test('streams every row from the stream handler, renamed from aliases to members', async () => {
    const rows = Array.from({ length: 70_000 }, (_, i) => ({ orders__status: `s${i % 3}`, orders__count: i }));
    const streamHandler = piping(() => Readable.from(rows));
    const queue = createQueue(streamHandler);

    const payload = {
      query: 'SELECT all',
      aliasNameToMember: { orders__status: 'Orders.status', orders__count: 'Orders.count' },
    };
    const streamed = await readAll(await queue.executeInQueue('stream', key('SELECT all'), payload));

    expect(streamed).toHaveLength(70_000);
    expect(streamed[0]).toEqual({ 'Orders.status': 's0', 'Orders.count': 0 });
    expect(streamed[69_999]).toEqual({ 'Orders.status': 's0', 'Orders.count': 69_999 });
    expect(streamHandler).toHaveBeenCalledWith(payload, expect.anything());
  });

  test('raises an error that follows some rows on the stream', async () => {
    const queue = createQueue(piping(() => Readable.from((function* rows() {
      yield { a: 1 };
      throw new Error('failed mid-stream');
    }()))));

    const result = await queue.executeInQueue('stream', key('SELECT a'), {});

    await expect(readAll(result)).rejects.toThrow('failed mid-stream');
  });

  test('raises a failing stream handler on the stream', async () => {
    const queue = createQueue(async () => {
      throw new Error('Table not found');
    });

    const result = await queue.executeInQueue('stream', key('SELECT bad'), {});

    await expect(readAll(result)).rejects.toThrow('Table not found');
  });

  test('forgets the stream once it is done', async () => {
    const queue = createQueue(piping(() => Readable.from([{ a: 1 }])));

    const k = key('SELECT a');
    await readAll(await queue.executeInQueue('stream', k, {}));
    await new Promise((resolve) => { setImmediate(resolve); });

    expect(queue.getQueryStream(queue.redisHash(k))).toBeUndefined();
  });

  test('fails without a stream handler', async () => {
    const queue = createQueue();

    await expect(queue.executeInQueue('stream', key('SELECT a'), {}))
      .rejects.toThrow('Streaming queries aren\'t supported by this queue');
  });
});
