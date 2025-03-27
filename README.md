# Attach to Serial Terminal
`screen /dev/ttyACM0 115200`

# Deploy to Board
`cargo flash --chip STM32F401RETx --release`
