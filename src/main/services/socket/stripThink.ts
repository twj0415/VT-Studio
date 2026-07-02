const THINK_OPEN = '<think>';
const THINK_CLOSE = '</think>';

type ThinkMode = 'content' | 'thinking';

export interface ThinkParseResult {
  content: string;
  thinking: string;
}

function getPartialTagLength(text: string, tag: string): number {
  const max = Math.min(text.length, tag.length - 1);

  for (let length = max; length > 0; length -= 1) {
    if (tag.startsWith(text.slice(-length))) {
      return length;
    }
  }

  return 0;
}

export class ThinkStreamParser {
  private mode: ThinkMode = 'content';
  private pending = '';

  feed(chunk: string): ThinkParseResult {
    let input = this.pending + chunk;
    this.pending = '';

    let content = '';
    let thinking = '';

    while (input.length > 0) {
      if (this.mode === 'content') {
        const openIndex = input.indexOf(THINK_OPEN);
        if (openIndex >= 0) {
          content += input.slice(0, openIndex);
          input = input.slice(openIndex + THINK_OPEN.length);
          this.mode = 'thinking';
          continue;
        }

        const partialLength = getPartialTagLength(input, THINK_OPEN);
        if (partialLength > 0) {
          content += input.slice(0, -partialLength);
          this.pending = input.slice(-partialLength);
        } else {
          content += input;
        }
        input = '';
        continue;
      }

      const closeIndex = input.indexOf(THINK_CLOSE);
      if (closeIndex >= 0) {
        thinking += input.slice(0, closeIndex);
        input = input.slice(closeIndex + THINK_CLOSE.length);
        this.mode = 'content';
        continue;
      }

      const partialLength = getPartialTagLength(input, THINK_CLOSE);
      if (partialLength > 0) {
        thinking += input.slice(0, -partialLength);
        this.pending = input.slice(-partialLength);
      } else {
        thinking += input;
      }
      input = '';
    }

    return { content, thinking };
  }

  flush(): ThinkParseResult {
    if (!this.pending) {
      return { content: '', thinking: '' };
    }

    const result = this.mode === 'thinking'
      ? { content: '', thinking: this.pending }
      : { content: this.pending, thinking: '' };
    this.pending = '';
    return result;
  }
}

export function stripThink(input: string): string {
  return input.replace(/<think>[\s\S]*?<\/think>/g, '').trim();
}
