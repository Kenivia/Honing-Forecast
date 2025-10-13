

type Props = {
    label: string,

    checked: boolean,
    setChecked: React.Dispatch<React.SetStateAction<boolean>>,
    accentColor?: string,
    textColor?: string
}

export default function LabeledCheckbox({ label, checked, setChecked, accentColor = "var(--control-checked-bg)", textColor = "--text-primary" }: Props) {
    return (
        <label style={{ display: "inline-flex", alignItems: "center", gap: 8, cursor: "pointer" }}>
            <input
                type="checkbox"
                checked={checked}
                onChange={(e) => setChecked(e.target.checked)}
                style={{ width: 16, height: 16, accentColor }}
            />
            <span style={{ color: textColor, fontSize: 'var(--font-size-sm)', userSelect: 'none' }}>
                {label}
            </span>
        </label>
    )
}