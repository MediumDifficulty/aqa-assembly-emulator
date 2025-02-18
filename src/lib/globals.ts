import { writable } from "svelte/store";

const RAM_SIZE = 256 * 4;

export const RAM = writable(new Uint8Array(RAM_SIZE))

export enum NumberFormat {
    Hex,
    Binary,
    SDecimal,
    UDecimal
}

export const NUMBER_FORMAT = writable(NumberFormat.Hex)
