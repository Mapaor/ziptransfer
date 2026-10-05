export function trackError(identifier, err, metadata) {
  try {
    console.error(identifier)
    console.error(err)
  } catch {
    // Silently ignore tracking failures
  }
}
