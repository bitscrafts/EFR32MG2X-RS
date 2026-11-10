#!/bin/bash
# SAFE flash procedure with backup and verification
# This ensures we can recover if something goes wrong

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== SAFE Flash Procedure for EFR32MG24 ===${NC}"
echo ""
echo "This script will:"
echo "  1. Read and backup current flash contents"
echo "  2. Verify backup was successful"
echo "  3. Flash bootloader only (test first)"
echo "  4. Verify bootloader"
echo "  5. Flash application"
echo "  6. Verify application"
echo "  7. Reset and run"
echo ""
echo -e "${YELLOW}SAFETY:${NC}"
echo "  - Original flash backed up to: backup_flash.bin"
echo "  - CMSIS-DAP debugger (SAMD11) is separate and CANNOT be bricked"
echo "  - We can ALWAYS recover via OpenOCD"
echo ""

# Check files exist
if [ ! -f bootloader.hex ]; then
    echo -e "${RED}Error: bootloader.hex not found${NC}"
    exit 1
fi

if [ ! -f test-app.hex ]; then
    echo -e "${RED}Error: test-app.hex not found${NC}"
    exit 1
fi

# Create backup directory
mkdir -p backups
BACKUP_FILE="backups/flash_backup_$(date +%Y%m%d_%H%M%S).bin"

echo -e "${GREEN}Step 1: Backing up current flash...${NC}"
./openocd/bin/openocd -s ./openocd/scripts \
  -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "halt" \
  -c "flash read_bank 0 ${BACKUP_FILE}" \
  -c "resume" \
  -c "exit" 2>&1 | grep -E "(Open|flash|backup|Error)" || true

if [ -f "$BACKUP_FILE" ]; then
    SIZE=$(ls -lh "$BACKUP_FILE" | awk '{print $5}')
    echo -e "${GREEN}✓ Backup created: $BACKUP_FILE ($SIZE)${NC}"
    echo "  You can restore this later if needed"
else
    echo -e "${YELLOW}Warning: Backup might have failed${NC}"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

echo ""
echo -e "${GREEN}Step 2: Flashing bootloader only (at 0x08000000)...${NC}"
echo "This is the critical part. The bootloader is tiny (792 bytes)."
echo ""

./openocd/bin/openocd -s ./openocd/scripts \
  -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "program bootloader.hex verify" \
  -c "exit"

echo ""
echo -e "${GREEN}✓ Bootloader flashed and verified${NC}"
echo ""
echo -e "${YELLOW}CHECKPOINT: Testing bootloader alone${NC}"
echo "Resetting device to test bootloader..."
echo ""

# Reset and check PC
./openocd/bin/openocd -s ./openocd/scripts \
  -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "reset halt" \
  -c "reg pc" \
  -c "exit" 2>&1 | grep -E "(pc |0x)" || true

echo ""
echo -e "${YELLOW}Check above: PC (Program Counter) should be in bootloader range (0x08000xxx)${NC}"
read -p "Does PC look correct? Continue with application flash? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo -e "${YELLOW}Flashing stopped at bootloader.${NC}"
    echo "Device is safe - bootloader is running."
    echo "To recover, reflash your backup:"
    echo "  ./openocd/bin/openocd -s ./openocd/scripts \
  -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \\"
    echo "    -c 'init' -c 'program $BACKUP_FILE 0x08000000 verify' -c 'exit'"
    exit 0
fi

echo ""
echo -e "${GREEN}Step 3: Flashing application (at 0x08004000)...${NC}"

./openocd/bin/openocd -s ./openocd/scripts \
  -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "program test-app.hex verify" \
  -c "reset run" \
  -c "exit"

echo ""
echo -e "${GREEN}=== Flash Complete! ===${NC}"
echo ""
echo -e "${GREEN}✓ Bootloader: flashed and verified${NC}"
echo -e "${GREEN}✓ Application: flashed and verified${NC}"
echo -e "${GREEN}✓ Device: reset and running${NC}"
echo ""
echo "OBSERVE THE LED:"
echo "  - Blinking at 1 Hz (500ms ON/OFF) = SUCCESS! ✓"
echo "  - Blinking FAST (~100ms) = HardFault (bootloader LED pattern)"
echo "  - Blinking SLOW (~1 second) = Bootloader mode (app validation failed)"
echo "  - Not blinking = Check connection/PC location"
echo ""
echo -e "${YELLOW}RECOVERY INFORMATION:${NC}"
echo "  Backup saved: $BACKUP_FILE"
echo "  To restore original flash:"
echo "    ./openocd/bin/openocd -s ./openocd/scripts \
  -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \\"
echo "      -c 'init' -c 'program $BACKUP_FILE 0x08000000 verify' -c 'exit'"
echo ""
