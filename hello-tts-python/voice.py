from dataclasses import dataclass


@dataclass
class Voice:
    name: str
    display_name: str = ''
    locale: str = ''
    gender: str = 'unknown'
