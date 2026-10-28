import init from '@uploadnt/core';

export * from '@uploadnt/core';

let ready: Promise<unknown> | undefined;

export const loadCore = () => (ready ??= init());
