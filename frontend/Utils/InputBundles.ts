import type { Dispatch, SetStateAction } from "react"

export type InputsValues = {
    mats: {
        owned: any
        prices: any
        leftover: any
    }
    juice: {
        weapon: {
            owned: any
            prices: any
            leftover: any
        }
        armor: {
            owned: any
            prices: any
            leftover: any
        }
    }
}

export type InputsSetters = {
    mats: {
        setOwned: Dispatch<SetStateAction<any>>
        setPrices: Dispatch<SetStateAction<any>>
        setLeftover: Dispatch<SetStateAction<any>>
    }
    juice: {
        weapon: {
            setOwned: Dispatch<SetStateAction<any>>
            setPrices: Dispatch<SetStateAction<any>>
            setLeftover: Dispatch<SetStateAction<any>>
        }
        armor: {
            setOwned: Dispatch<SetStateAction<any>>
            setPrices: Dispatch<SetStateAction<any>>
            setLeftover: Dispatch<SetStateAction<any>>
        }
    }
}

export type InputsBundle = {
    values: InputsValues
    setters?: InputsSetters
}

export type InputsBundleWithSetters = {
    values: InputsValues
    setters: InputsSetters
}
