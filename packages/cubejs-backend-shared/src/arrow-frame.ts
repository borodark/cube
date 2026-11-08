/**
 * Marks an object in a query's row stream that carries a frame of a Cube Store
 * result, as Arrow, in place of the rows it holds. Only a stream whose reader
 * asked for frames carries them (`arrowFrames` on the query), and only that
 * reader, the SQL API, reads them.
 *
 * `Symbol.for`, so every copy of this package agrees on it.
 */
export const ARROW_FRAME = Symbol.for('cubejs.arrowFrame');

export type ArrowFrame = {
  // The whole message Cube Store sent the frame in.
  data: Buffer;
  // How the rows the frame stands for would have been keyed: by the member each
  // column's alias stands for. Set by the stream that renames rows that way.
  aliasNameToMember?: Record<string, string> | null;
};

export type ArrowFrameObject = { [ARROW_FRAME]: ArrowFrame };

export function arrowFrameObject(frame: ArrowFrame): ArrowFrameObject {
  return { [ARROW_FRAME]: frame };
}

/**
 * The frame `value` carries, if it is a frame object rather than a row.
 */
export function getArrowFrame(value: unknown): ArrowFrame | undefined {
  return (value as Partial<ArrowFrameObject> | null)?.[ARROW_FRAME];
}
