const statusToneClassMap: Record<string, string> = {
  'status.pending': 'st-pending',
  'status.running': 'st-running',
  'status.retrying': 'st-retrying',
  'status.waiting_user': 'st-waiting',
  'status.succeeded': 'st-succeeded',
  'status.failed': 'st-failed',
  'status.cancelled': 'st-cancelled',
  'status.skipped': 'st-skipped',
}

export function getStatusToneClass(colorToken?: string) {
  return colorToken ? statusToneClassMap[colorToken] ?? 'st-pending' : 'st-pending'
}
