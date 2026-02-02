import { IconMap } from "@/Utils/Constants.ts"
import React from "react"

interface IconProps {
    iconName: string
    size?: number
    style?: React.CSSProperties
    display_text?: string | null
    display_text_right?: string | null
}

export default function Icon({ iconName: name, display_text: display_text = null, display_text_right = null, size = 20, style }: IconProps) {
    const iconPath = IconMap[name]

    if (!iconPath) {
        return <span style={style}>{name}</span>
    }

    return (
        <div style={{ display: "flex", alignItems: "center", gap: "6px", ...style }}>
            <span>{display_text === null ? name : display_text}</span>
            <img
                src={iconPath}
                alt={name}
                style={{
                    width: size,
                    height: size,
                    objectFit: "contain",
                }}
                onError={(e) => {
                    // Fallback to text if image fails to load
                    console.log("EquipmentIcon: Error loading image from " + iconPath, e)
                    const target = e.target as HTMLImageElement
                    target.style.display = "none"
                    const parent = target.parentElement
                    if (parent) {
                        parent.innerHTML = name
                    }
                }}
            />
            <span>{display_text_right === null ? "" : display_text_right}</span>
        </div>
    )
}
