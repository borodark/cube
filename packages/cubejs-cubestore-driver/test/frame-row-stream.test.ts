import { parseCubestoreResultMessage } from '@cubejs-backend/native';

import { getArrowFrame } from '@cubejs-backend/shared';

import { FrameRowStream } from '../src/FrameRowStream';

// Decoding a frame is the native addon's job; here a frame is the JSON of its rows.
// Like the native result, the parsed frame reads as its rows when iterated, and
// its raw data is a handle to them, not the rows.
jest.mock('@cubejs-backend/native', () => ({
  parseCubestoreResultMessage: jest.fn(async (frame: Buffer) => {
    const rows = JSON.parse(frame.toString());
    if (rows === 'bad') {
      throw new Error('not an Arrow frame');
    }
    return {
      [Symbol.iterator]: () => rows[Symbol.iterator](),
      getRawData: () => [{ nativeHandle: true }],
    };
  }),
}));

const parse = parseCubestoreResultMessage as jest.Mock;

const frame = (from: number, count: number) => Buffer.from(JSON.stringify(
  Array.from({ length: count }, (_, i) => ({ n: from + i }))
));

const readAll = async (stream: AsyncIterable<unknown>) => {
  const rows: unknown[] = [];

  // eslint-disable-next-line no-restricted-syntax
  for await (const row of stream) {
    rows.push(row);
  }
  return rows;
};

const tick = () => new Promise((resolve) => { setImmediate(resolve); });

describe('FrameRowStream', () => {
  beforeEach(() => parse.mockClear());

  it('reads the rows of every frame in order and ends after the last', async () => {
    const stream = new FrameRowStream();
    const rows = readAll(stream);

    stream.pushFrame(frame(0, 3), false);
    await tick();
    stream.pushFrame(frame(3, 2), false);
    stream.pushFrame(frame(5, 1), true);

    expect(await rows).toEqual([0, 1, 2, 3, 4, 5].map((n) => ({ n })));
  });

  it('ends after an empty last frame', async () => {
    const stream = new FrameRowStream();
    stream.pushFrame(frame(0, 0), true);

    expect(await readAll(stream)).toEqual([]);
  });

  it('decodes a frame only once the reader wants more rows', async () => {
    const stream = new FrameRowStream();

    for (let i = 0; i < 10; i++) {
      stream.pushFrame(frame(i * 100, 100), i === 9);
    }
    await tick();
    // Nobody reads yet, so nothing is decoded.
    expect(parse).not.toHaveBeenCalled();

    // A frame's rows overfill the buffer, so one row takes one frame.
    stream.read();
    await tick();
    expect(parse).toHaveBeenCalledTimes(1);

    expect(await readAll(stream)).toHaveLength(1000);
    expect(parse).toHaveBeenCalledTimes(10);
  });

  it('fails with an error decoding a frame', async () => {
    const stream = new FrameRowStream();
    stream.pushFrame(frame(0, 2), false);
    stream.pushFrame(Buffer.from('"bad"'), true);

    await expect(readAll(stream)).rejects.toThrow('not an Arrow frame');
  });

  it('hands over the frames themselves, in order, to a reader that takes them', async () => {
    const stream = new FrameRowStream({ frames: true });
    const read = readAll(stream);
    stream.pushFrame(frame(0, 2), false);
    await tick();
    stream.pushFrame(frame(2, 1), true);

    const frames = (await read).map((object) => getArrowFrame(object)?.data);

    expect(frames).toEqual([frame(0, 2), frame(2, 1)]);
    expect(parse).not.toHaveBeenCalled();
  });

  it('drops frames that arrive after the reader gave up', async () => {
    const stream = new FrameRowStream();
    stream.pushFrame(frame(0, 1), false);
    stream.destroy();
    stream.pushFrame(frame(1, 1), true);
    await tick();

    expect(parse).not.toHaveBeenCalled();
  });
});
