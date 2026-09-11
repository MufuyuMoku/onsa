/**
 * The commands the interface may call (SPEC section 2).
 *
 * The interface is a remote control: everything it knows comes from here, and
 * every failure arrives as a stable code that the dictionaries translate.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';
import type { MessageKey } from '$lib/i18n/dictionary';
import type { Theme } from '$lib/theme/types';

/** Name, version and starting theme of the running application. */
export interface AppInfo {
	name: string;
	version: string;
	defaultThemeId: string;
}

/** A failure the backend reported, already turned into a dictionary key. */
export class BackendError extends Error {
	readonly messageKey: MessageKey;

	constructor(messageKey: MessageKey, cause?: unknown) {
		super(messageKey, { cause });
		this.name = 'BackendError';
		this.messageKey = messageKey;
	}
}

/** Whether the interface is running inside the Onsa window. */
export function hasBackend(): boolean {
	return isTauri();
}

/** Asks the backend what it is. */
export function appInfo(): Promise<AppInfo> {
	return call<AppInfo>('app_info');
}

/** Lists the themes that can be chosen. */
export function themeList(): Promise<Theme[]> {
	return call<Theme[]>('theme_list');
}

/** Reads one theme by identifier. */
export function themeGet(id: string): Promise<Theme> {
	return call<Theme>('theme_get', { id });
}

/** The error codes the backend can send, mapped to dictionary keys. */
const ERROR_KEYS: Record<string, MessageKey> = {
	theme_not_found: 'error.theme_not_found'
};

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	if (!isTauri()) throw new BackendError('error.backend');
	try {
		return await invoke<T>(command, args);
	} catch (cause) {
		throw new BackendError(errorKey(cause), cause);
	}
}

function errorKey(cause: unknown): MessageKey {
	if (typeof cause === 'object' && cause !== null && 'code' in cause) {
		const code = (cause as { code: unknown }).code;
		if (typeof code === 'string' && code in ERROR_KEYS) return ERROR_KEYS[code]!;
	}
	return 'error.backend';
}
