type CommandResult<T> =
  { status: "ok"; data: T } | { status: "error"; error: { message: string } };

export function unwrap<T>(result: CommandResult<T>): T {
  if (result.status === "error") {
    throw new Error(result.error.message);
  }
  return result.data;
}
