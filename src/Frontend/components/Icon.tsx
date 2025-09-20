import React from 'react'

interface IconProps {
    iconName: string
    size?: number
    style?: React.CSSProperties
    display_text?: string | null
}

const IconMap: Record<string, string> = {
    'Helmet': '/Honing-Forecast/Icons/Equipments/Helmet.webp',
    'Shoulder': '/Honing-Forecast/Icons/Equipments/Shoulder.webp',
    'Chest': '/Honing-Forecast/Icons/Equipments/Chest.webp',
    'Pants': '/Honing-Forecast/Icons/Equipments/Pants.webp',
    'Glove': '/Honing-Forecast/Icons/Equipments/Gloves.webp',
    'Weapon': '/Honing-Forecast/Icons/Equipments/Weapon.webp',
    'Red': '/Honing-Forecast/Icons/Materials/Red.webp',
    'Blue': '/Honing-Forecast/Icons/Materials/Blue.webp',
    'Leaps': '/Honing-Forecast/Icons/Materials/Leap.webp',
    'Shards': '/Honing-Forecast/Icons/Materials/Shard.webp',
    'Oreha': '/Honing-Forecast/Icons/Materials/Oreha.webp',
    'Gold': '/Honing-Forecast/Icons/Materials/Gold.webp',
    'Silver(WIP)': '/Honing-Forecast/Icons/Materials/Silver.webp',
    'Red juice': '/Honing-Forecast/Icons/Materials/Red juice.webp',
    'Blue juice': '/Honing-Forecast/Icons/Materials/Blue juice.webp',
    'Special leaps': '/Honing-Forecast/Icons/Materials/Special leap.webp',
}

export default function Icon({ iconName: name, display_text: display_text = null, size = 20, style }: IconProps) {
    const iconPath = IconMap[name]

    if (!iconPath) {
        return <span style={style}>{name}</span>
    }

    return (
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px', ...style }}>
            <span>{display_text === null ? name : display_text}</span>
            <img
                src={iconPath}
                alt={name}
                style={{
                    width: size,
                    height: size,
                    objectFit: 'contain'
                }}
                onError={(e) => {
                    // Fallback to text if image fails to load
                    console.log("EquipmentIcon: Error loading image from " + iconPath, e)
                    const target = e.target as HTMLImageElement
                    target.style.display = 'none'
                    const parent = target.parentElement
                    if (parent) {
                        parent.innerHTML = name
                    }
                }}
            />

        </div>
    )
}

