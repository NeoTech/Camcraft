## #3089 - Name the operations that need the missing machine

Include the affected operations in the shared default-machine refusal. The
verification pass already knows which setups and operations use that machine.
Listing them in posting order gives the operator the setup, operation and file
to check. Moving the check into each output mode would duplicate the refusal.
A future change to the wording only needs to update this diagnostic.
