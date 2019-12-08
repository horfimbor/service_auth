class Auth extends HTMLElement {
    constructor() {
        super();

        this.attachShadow({ mode: 'open' });
        this.shadowRoot.innerHTML = `
            <form action="http://localhost:8000" method="post">
                <input minlength="5" placeholder="passphrase" type="password" name="passphrase">
                <input type="submit" value="generate"/>
            </form>
        `
    }
}
customElements.define('hf-auth-login', Auth);