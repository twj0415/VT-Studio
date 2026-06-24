let seed = 1

export function createMockId(prefix: string) {
  seed += 1
  return `${prefix}_${seed.toString().padStart(4, '0')}`
}
