class Signup extends HTMLElement {
    constructor() {
        super();

        let passphrase = this.getAttribute('passphrase')

        this.attachShadow({ mode: 'open' });
        this.shadowRoot.innerHTML = `
            <form class="signup" method="post">
            <p>Ce compte n'existe pas, voulez-vous le créer ?</p>
                <input minlength="1" placeholder="name" type="text" name="text" class="name">
                <input type="hidden" value="${passphrase}" class="passphrase" />
                <input type="submit" value="signup"/>
            </form>
        `
    }

    connectedCallback() {
        this.shadowRoot.querySelector("form.signup").addEventListener('submit', this._signup.bind(this));
    }

    disconnectedCallback() {
        this.shadowRoot.querySelector("form.signup").removeEventListener('submit', this._signup);
    }

    _signup(e) {
        e.preventDefault();

        console.debug(this.shadowRoot.querySelector(".name").value)

        fetch("http://localhost:8000/signup", {
                method : "POST",
                headers: {
                   cache : "no-cache"
                },
                body : JSON.stringify({
                    name: this.shadowRoot.querySelector(".name").value,
                    passphrase: this.shadowRoot.querySelector(".passphrase").value
                }),
            })
            .then(res => res.text()) // parse response as JSON with res.json
            .then(response => {
                    var event = new CustomEvent('_auth_login', { 'detail': response });
                    document.dispatchEvent(event);
             })
            .catch(err => {
                console.log({service:"auth", status:"KO", error:err})
                alert("sorry, cannot signup")
            });
    }
}
customElements.define('hf-auth-signup', Signup);