import { writable } from "svelte/store";

export const RAM_SIZE = 256 * 4;

export const RAM = writable(new Uint8Array(RAM_SIZE))
export const REGISTERS = writable(new Uint32Array(16))
export const FLAGS = writable(0)

export enum NumberFormat {
    Hex,
    Binary,
    SDecimal,
    UDecimal
}

export const NUMBER_FORMAT = writable(NumberFormat.Hex)
