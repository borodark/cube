import { Readable } from 'stream';
import { parseCubestoreResultMessage, ResultRow } from '@cubejs-backend/native';
import { arrowFrameObject } from '@cubejs-backend/shared';

/**
 * The rows of a streamed Cube Store result, read frame by frame as they arrive.
 *
 * A frame stays a compact Arrow message until the reader asks for more rows:
 * only then is it decoded, one frame at a time, so a slow reader holds frames
 * rather than every row of them.
 *
 * With `frames` set, a reader that takes Arrow (the SQL API) gets each frame
 * as it is instead, marked as a frame object (`ARROW_FRAME`), and nothing is
 * decoded here at all.
 */
export class FrameRowStream extends Readable {
  protected readonly frames: Buffer[] = [];

  protected lastFrameReceived = false;

  protected decoding = false;

  protected wanted = false;

  public constructor(protected readonly options: { frames?: boolean } = {}) {
    super({ objectMode: true });
  }

  /**
   * Adds the next frame of the result. Frames must come in order. Once the
   * reader has given up on the stream they are dropped: Cube Store has no way
   * to be told to stop, so the rest of the result still arrives.
   */
  public pushFrame(frame: Buffer, isLast: boolean) {
    if (this.destroyed) {
      return;
    }
    this.frames.push(frame);
    if (isLast) {
      this.lastFrameReceived = true;
    }
    if (this.wanted) {
      this.decodeNext();
    }
  }

  public _read() {
    this.wanted = true;
    this.decodeNext();
  }

  public _destroy(error: Error | null, callback: (error?: Error | null) => void) {
    this.frames.length = 0;
    callback(error);
  }

  protected decodeNext() {
    if (this.decoding || this.destroyed) {
      return;
    }

    const frame = this.frames.shift();
    if (!frame) {
      if (this.lastFrameReceived) {
        this.push(null);
      }
      return;
    }

    if (this.options.frames) {
      this.wanted = false;
      if (this.push(arrowFrameObject({ data: frame }))) {
        this.wanted = true;
        this.decodeNext();
      }

      return;
    }

    this.decoding = true;
    this.wanted = false;
    parseCubestoreResultMessage(frame).then(
      (result) => {
        this.decoding = false;
        if (this.destroyed) {
          return;
        }

        let more = true;

        // The result reads as its rows; `getRawData()` is only the native handle
        // eslint-disable-next-line no-restricted-syntax
        for (const row of result as unknown as Iterable<ResultRow>) {
          more = this.push(row);
        }

        // A full buffer calls `_read` again once the reader drains it.
        if (more) {
          this.wanted = true;
          this.decodeNext();
        }
      },
      (error) => {
        this.decoding = false;
        this.destroy(error);
      }
    );
  }
}
