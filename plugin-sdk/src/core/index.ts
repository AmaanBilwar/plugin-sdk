export type MaybePromise<T> = T | Promise<T>;

export type Factory<TContext, TOptions, TResult> = (
  context: TContext,
  options?: TOptions,
) => MaybePromise<TResult>;

export type RuntimePart<TRuntime> = (runtime: TRuntime) => MaybePromise<void>;

export type ValueOrFactory<TValue, TArgs extends unknown[]> =
  | TValue
  | ((...args: TArgs) => MaybePromise<TValue>);

export async function runFactories<TContext, TOptions, TResult>(
  factories: Array<Factory<TContext, TOptions, TResult>>,
  context: TContext,
  options: TOptions | undefined,
): Promise<TResult[]> {
  return Promise.all(factories.map((factory) => factory(context, options)));
}

export async function runRuntimeParts<TRuntime>(
  parts: Array<RuntimePart<TRuntime>>,
  runtime: TRuntime,
): Promise<void> {
  for (const part of parts) {
    await part(runtime);
  }
}

export async function resolveValueOrFactory<TValue, TArgs extends unknown[]>(
  input: ValueOrFactory<TValue, TArgs>,
  ...args: TArgs
): Promise<TValue> {
  if (typeof input === "function") {
    return (input as (...innerArgs: TArgs) => MaybePromise<TValue>)(...args);
  }
  return input;
}

export function matchesAnyPattern(value: string, patterns: Array<string | RegExp>): boolean {
  return patterns.some((pattern) =>
    typeof pattern === "string" ? value.includes(pattern) : pattern.test(value),
  );
}
