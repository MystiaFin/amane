#include <security/pam_appl.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int pam_sm_authenticate(pam_handle_t *handle, int flags, int count, const char **args) {
    (void)flags;
    const char *mode = count ? args[0] : "interactive";
    if (!strcmp(mode, "denied")) return PAM_AUTH_ERR;
    if (!strcmp(mode, "error")) return PAM_SYSTEM_ERR;
    if (!strcmp(mode, "change-user")) return pam_set_item(handle, PAM_USER, "different-user");
    if (!strcmp(mode, "check-user")) {
        const void *user = NULL;
        int status = pam_get_item(handle, PAM_USER, &user);
        if (status != PAM_SUCCESS) return status;
        return user && !strcmp(user, "amane-test-user") ? PAM_SUCCESS : PAM_AUTH_ERR;
    }
    if (!strcmp(mode, "success") || !strcmp(mode, "account-denied")) return PAM_SUCCESS;

    const void *item = NULL;
    int status = pam_get_item(handle, PAM_CONV, &item);
    if (status != PAM_SUCCESS) return status;
    const struct pam_conv *conversation = item;
    struct pam_message info = {PAM_TEXT_INFO, "Read this"};
    struct pam_message warning = {PAM_ERROR_MSG, "Example warning"};
    struct pam_message password = {PAM_PROMPT_ECHO_OFF, "Password:"};
    struct pam_message code = {PAM_PROMPT_ECHO_ON, "Code:"};
    struct pam_message finishing = {PAM_TEXT_INFO, "Finishing"};
    const struct pam_message *messages[] = {&info, &warning, &password, &code};
    struct pam_response *responses = NULL;
    if (!strcmp(mode, "unknown-style")) {
        struct pam_message unknown = {99, "Unsupported"};
        const struct pam_message *message = &unknown;
        status = conversation->conv(1, &message, &responses, conversation->appdata_ptr);
        free(responses);
        return status;
    }
    if (!strcmp(mode, "delayed")) {
        const struct pam_message *message = &finishing;
        status = conversation->conv(1, &message, &responses, conversation->appdata_ptr);
        free(responses);
        if (count < 2) return PAM_SYSTEM_ERR;
        while (access(args[1], F_OK)) usleep(1000);
        return status;
    }
    int password_only = !strcmp(mode, "password-only");
    if (password_only) code.msg_style = PAM_PROMPT_ECHO_OFF;
    status = conversation->conv(4, messages, &responses, conversation->appdata_ptr);
    if (status != PAM_SUCCESS) return status;
    int correct = responses[0].resp == NULL && responses[1].resp == NULL &&
        responses[2].resp && !strcmp(responses[2].resp, "test-password") &&
        responses[3].resp && !strcmp(responses[3].resp, password_only ? "test-password" : "246810");
    for (int index = 0; index < 4; index++) free(responses[index].resp);
    free(responses);
    return correct ? PAM_SUCCESS : PAM_AUTH_ERR;
}

int pam_sm_acct_mgmt(pam_handle_t *handle, int flags, int count, const char **args) {
    (void)handle;
    (void)flags;
    return count && !strcmp(args[0], "account-denied") ? PAM_ACCT_EXPIRED : PAM_SUCCESS;
}

int pam_sm_setcred(pam_handle_t *handle, int flags, int count, const char **args) {
    (void)handle;
    (void)flags;
    (void)count;
    (void)args;
    return PAM_SUCCESS;
}
