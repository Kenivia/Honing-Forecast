import React from "react"

interface IconProps {
    iconName: string
    size?: number
    style?: React.CSSProperties
    display_text?: string | null
    display_text_right?: string | null
}

const IconMap: Record<string, string> = {
    Helmet: "/Honing-Forecast/Icons/Equipments/Helmet.webp",
    Shoulder: "/Honing-Forecast/Icons/Equipments/Shoulder.webp",
    Chest: "/Honing-Forecast/Icons/Equipments/Chest.webp",
    Pants: "/Honing-Forecast/Icons/Equipments/Pants.webp",
    Glove: "/Honing-Forecast/Icons/Equipments/Gloves.webp",
    Weapon: "/Honing-Forecast/Icons/Equipments/Weapon.webp",
    Red: "/Honing-Forecast/Icons/Materials/Red.webp",
    Blue: "/Honing-Forecast/Icons/Materials/Blue.webp",
    Leaps: "/Honing-Forecast/Icons/Materials/Leap.webp",
    Shards: "/Honing-Forecast/Icons/Materials/Shard.webp",
    Oreha: "/Honing-Forecast/Icons/Materials/Oreha.webp",
    Gold: "/Honing-Forecast/Icons/Materials/Gold.webp",
    Silver: "/Honing-Forecast/Icons/Materials/Silver.webp",
    "Lava's Breath": "/Honing-Forecast/Icons/Materials/Lava's Breath.webp",
    "Glacier's Breath": "/Honing-Forecast/Icons/Materials/Glacier's Breath.webp",
    "Special Leap": "/Honing-Forecast/Icons/Materials/Special Leap.webp",

    "11-14 Armor": "/Honing-Forecast/Icons/Materials/Armor Book.webp",
    "11-14 Weapon": "/Honing-Forecast/Icons/Materials/Weapon Book.webp",

    "15-18 Armor": "/Honing-Forecast/Icons/Materials/Armor Book.webp",
    "15-18 Weapon": "/Honing-Forecast/Icons/Materials/Weapon Book.webp",

    "19-20 Armor": "/Honing-Forecast/Icons/Materials/Armor Book.webp",
    "19-20 Weapon": "/Honing-Forecast/Icons/Materials/Weapon Book.webp",

    "Forecast Icon": "/Honing-Forecast/forecast icon.webp",
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
