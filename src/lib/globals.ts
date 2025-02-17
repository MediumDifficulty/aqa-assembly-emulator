import { writable } from "svelte/store";

const RAM_SIZE = 16 * 4;

export const RAM = writable(new Uint8Array(RAM_SIZE))